use crate::web::{Pool, Session, Request, Response, ApiResponse};
use crate::web::from::{Path, Json};
use crate::web::data::{Int, UInt, String as Str};
use serde::{Deserialize, Serialize};
use serde_json::json;
use actix_files::NamedFile;
use std::path::PathBuf;

// ─── Structs ──────────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct QuizRow {
    pub id: UInt,
    pub user_id: UInt,
    pub creator_name: Str,
    pub title: Str,
    pub description: Option<Str>,
    pub question_count: Int,
    pub attempt_count: Int,
    pub is_published: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Serialize)]
pub struct QuizOptionRow {
    pub id: UInt,
    pub question_id: UInt,
    pub label: Str,
}

#[derive(Serialize)]
pub struct QuizQuestionRow {
    pub id: UInt,
    pub sort_order: u16,
    pub question: Str,
    pub r#type: Str,
    pub options: Vec<QuizOptionRow>,
}

#[derive(Serialize)]
pub struct LeaderboardRow {
    pub rank: Int,
    pub user_id: UInt,
    pub fullname: Str,
    pub username: Str,
    pub score: f64,
    pub finished_at: Option<chrono::NaiveDateTime>,
}

#[derive(Deserialize)]
pub struct CreateQuizPayload {
    pub title: Str,
    pub description: Option<Str>,
    pub questions: Vec<CreateQuestionPayload>,
}

#[derive(Deserialize)]
pub struct CreateQuestionPayload {
    pub question: Str,
    pub r#type: Str,     // "single" | "multiple" | "text"
    pub answer_key: Option<Str>,
    pub options: Vec<CreateOptionPayload>,
}

#[derive(Deserialize)]
pub struct CreateOptionPayload {
    pub label: Str,
    pub is_correct: bool,
}

#[derive(Deserialize)]
pub struct SubmitPayload {
    pub answers: Vec<SubmitAnswer>,
}

#[derive(Deserialize)]
pub struct SubmitAnswer {
    pub question_id: UInt,
    pub answer_text: Option<Str>,
    pub option_ids: Option<Vec<UInt>>,
}

// ─── Pages ────────────────────────────────────────────────────────────────────

pub async fn page_list(_req: Request) -> actix_web::Result<NamedFile> {
    let path: PathBuf = "./src/module/quiz/page_list.html".into();
    Ok(NamedFile::open(path)?)
}

pub async fn page_info(_req: Request) -> actix_web::Result<NamedFile> {
    let path: PathBuf = "./src/module/quiz/page_info.html".into();
    Ok(NamedFile::open(path)?)
}

pub async fn page_take(session: Session, _req: Request) -> Response {
    let _ = auth!(session);
    let path: PathBuf = "./src/module/quiz/page_take.html".into();
    match NamedFile::open(path) {
        Ok(file) => file.into_response(&_req),
        Err(_) => Response::InternalServerError().finish(),
    }
}

pub async fn page_create(session: Session, _req: Request) -> Response {
    let _ = auth!(session);
    let path: PathBuf = "./src/module/quiz/page_create.html".into();
    match NamedFile::open(path) {
        Ok(file) => file.into_response(&_req),
        Err(_) => Response::InternalServerError().finish(),
    }
}

// ─── API: Daftar Quiz ─────────────────────────────────────────────────────────

pub async fn api_list_quiz(pool: Pool) -> Response {
    let rows = sqlx::query_as::<_, (UInt, UInt, Str, Str, Option<Str>, i64, i64, i8, chrono::NaiveDateTime)>(
        r#"SELECT
            q.id, q.user_id, u.fullname, q.title, q.description,
            COUNT(DISTINCT qq.id) AS question_count,
            COUNT(DISTINCT qa.id) AS attempt_count,
            q.is_published, q.created_at
        FROM quiz q
        JOIN users u ON u.id = q.user_id
        LEFT JOIN quiz_question qq ON qq.quiz_id = q.id
        LEFT JOIN quiz_attempt qa ON qa.quiz_id = q.id AND qa.finished_at IS NOT NULL
        WHERE q.is_published = 1
        GROUP BY q.id
        ORDER BY q.created_at DESC
        LIMIT 50"#,
    )
    .fetch_all(pool.as_ref())
    .await;

    match rows {
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal memuat daftar quiz: {}", e),
            data: None,
            meta: None,
        }),
        Ok(rows) => {
            let list: Vec<_> = rows
                .into_iter()
                .map(|(id, user_id, creator_name, title, description, question_count, attempt_count, is_published, created_at)| {
                    json!({
                        "id": id,
                        "user_id": user_id,
                        "creator_name": creator_name,
                        "title": title,
                        "description": description,
                        "question_count": question_count,
                        "attempt_count": attempt_count,
                        "is_published": is_published == 1,
                        "created_at": created_at
                    })
                })
                .collect();
            Response::Ok().json(ApiResponse {
                success: true,
                message: "ok".into(),
                data: Some(json!(list)),
                meta: None,
            })
        }
    }
}

// ─── API: Info Quiz ───────────────────────────────────────────────────────────

pub async fn api_quiz_info(pool: Pool, path: Path<(UInt,)>) -> Response {
    let (quiz_id,) = path.into_inner();

    let row = sqlx::query_as::<_, (UInt, UInt, Str, Str, Option<Str>, i8, chrono::NaiveDateTime)>(
        r#"SELECT q.id, q.user_id, u.fullname, q.title, q.description, q.is_published, q.created_at
           FROM quiz q JOIN users u ON u.id = q.user_id
           WHERE q.id = ? AND q.is_published = 1"#,
    )
    .bind(quiz_id)
    .fetch_optional(pool.as_ref())
    .await;

    match row {
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Error: {}", e),
            data: None,
            meta: None,
        }),
        Ok(None) => Response::NotFound().json(ApiResponse {
            success: false,
            message: "Quiz tidak ditemukan".into(),
            data: None,
            meta: None,
        }),
        Ok(Some((id, user_id, creator_name, title, description, is_published, created_at))) => {
            // Hitung jumlah soal dan peserta
            let counts = sqlx::query_as::<_, (i64, i64)>(
                "SELECT COUNT(DISTINCT qq.id), COUNT(DISTINCT qa.id)
                 FROM quiz_question qq
                 LEFT JOIN quiz_attempt qa ON qa.quiz_id = qq.quiz_id AND qa.finished_at IS NOT NULL
                 WHERE qq.quiz_id = ?",
            )
            .bind(quiz_id)
            .fetch_one(pool.as_ref())
            .await;

            let (q_count, a_count) = counts.unwrap_or((0, 0));

            Response::Ok().json(ApiResponse {
                success: true,
                message: "ok".into(),
                data: Some(json!({
                    "id": id,
                    "user_id": user_id,
                    "creator_name": creator_name,
                    "title": title,
                    "description": description,
                    "is_published": is_published == 1,
                    "created_at": created_at,
                    "question_count": q_count,
                    "attempt_count": a_count,
                })),
                meta: None,
            })
        }
    }
}

// ─── API: Soal-soal Quiz ──────────────────────────────────────────────────────

pub async fn api_get_questions(pool: Pool, path: Path<(UInt,)>, session: Session) -> Response {
    let _ = auth!(session);
    let (quiz_id,) = path.into_inner();

    // Cek quiz exist dan published
    let exists = sqlx::query_as::<_, (UInt,)>(
        "SELECT id FROM quiz WHERE id = ? AND is_published = 1",
    )
    .bind(quiz_id)
    .fetch_optional(pool.as_ref())
    .await;

    match exists {
        Err(e) => return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Error: {}", e),
            data: None,
            meta: None,
        }),
        Ok(None) => return Response::NotFound().json(ApiResponse {
            success: false,
            message: "Quiz tidak ditemukan".into(),
            data: None,
            meta: None,
        }),
        Ok(Some(_)) => {}
    }

    // Ambil soal
    let questions = sqlx::query_as::<_, (UInt, u16, Str, Str)>(
        "SELECT id, sort_order, question, type FROM quiz_question
         WHERE quiz_id = ? ORDER BY sort_order ASC",
    )
    .bind(quiz_id)
    .fetch_all(pool.as_ref())
    .await;

    let questions = match questions {
        Err(e) => return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal memuat soal: {}", e),
            data: None,
            meta: None,
        }),
        Ok(q) => q,
    };

    // Ambil semua opsi untuk quiz ini sekaligus
    let options = sqlx::query_as::<_, (UInt, UInt, Str)>(
        "SELECT o.id, o.question_id, o.label
         FROM quiz_option o
         JOIN quiz_question q ON q.id = o.question_id
         WHERE q.quiz_id = ?
         ORDER BY o.id ASC",
    )
    .bind(quiz_id)
    .fetch_all(pool.as_ref())
    .await;

    let options = match options {
        Err(e) => return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal memuat opsi: {}", e),
            data: None,
            meta: None,
        }),
        Ok(o) => o,
    };

    // Gabungkan soal + opsi
    let result: Vec<_> = questions
        .into_iter()
        .map(|(id, sort_order, question, qtype)| {
            let opts: Vec<_> = options
                .iter()
                .filter(|(_, qid, _)| *qid == id)
                .map(|(oid, _, label)| json!({ "id": oid, "label": label }))
                .collect();
            json!({
                "id": id,
                "sort_order": sort_order,
                "question": question,
                "type": qtype,
                "options": opts,
            })
        })
        .collect();

    Response::Ok().json(ApiResponse {
        success: true,
        message: "ok".into(),
        data: Some(json!(result)),
        meta: None,
    })
}

// ─── API: Leaderboard ─────────────────────────────────────────────────────────

pub async fn api_leaderboard(pool: Pool, path: Path<(UInt,)>) -> Response {
    let (quiz_id,) = path.into_inner();

    let rows = sqlx::query_as::<_, (UInt, Str, Str, f64, Option<chrono::NaiveDateTime>)>(
        r#"SELECT u.id, u.fullname, u.username, a.score, a.finished_at
           FROM quiz_attempt a
           JOIN users u ON u.id = a.user_id
           WHERE a.quiz_id = ? AND a.finished_at IS NOT NULL AND a.score IS NOT NULL
           ORDER BY a.score DESC, a.finished_at ASC
           LIMIT 50"#,
    )
    .bind(quiz_id)
    .fetch_all(pool.as_ref())
    .await;

    match rows {
        Err(e) => Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Error: {}", e),
            data: None,
            meta: None,
        }),
        Ok(rows) => {
            let board: Vec<_> = rows
                .into_iter()
                .enumerate()
                .map(|(i, (uid, fullname, username, score, finished_at))| {
                    json!({
                        "rank": i + 1,
                        "user_id": uid,
                        "fullname": fullname,
                        "username": username,
                        "score": score,
                        "finished_at": finished_at,
                    })
                })
                .collect();
            Response::Ok().json(ApiResponse {
                success: true,
                message: "ok".into(),
                data: Some(json!(board)),
                meta: None,
            })
        }
    }
}

// ─── API: Submit Jawaban ──────────────────────────────────────────────────────

pub async fn api_submit(
    pool: Pool,
    session: Session,
    path: Path<(UInt,)>,
    body: Json<SubmitPayload>,
) -> Response {
    let user_id = auth!(session);
    let (quiz_id,) = path.into_inner();

    // Cek quiz ada
    let quiz_check = sqlx::query_as::<_, (UInt,)>(
        "SELECT id FROM quiz WHERE id = ? AND is_published = 1",
    )
    .bind(quiz_id)
    .fetch_optional(pool.as_ref())
    .await;

    match quiz_check {
        Err(e) => return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Error: {}", e),
            data: None,
            meta: None,
        }),
        Ok(None) => return Response::NotFound().json(ApiResponse {
            success: false,
            message: "Quiz tidak ditemukan".into(),
            data: None,
            meta: None,
        }),
        Ok(Some(_)) => {}
    }

    // Cek apakah sudah pernah submit
    let already = sqlx::query_as::<_, (UInt,)>(
        "SELECT id FROM quiz_attempt WHERE quiz_id = ? AND user_id = ? AND finished_at IS NOT NULL",
    )
    .bind(quiz_id)
    .bind(user_id)
    .fetch_optional(pool.as_ref())
    .await;

    if let Ok(Some(_)) = already {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Kamu sudah mengerjakan quiz ini".into(),
            data: None,
            meta: None,
        });
    }

    // Buat attempt baru
    let attempt_id = sqlx::query(
        "INSERT INTO quiz_attempt (quiz_id, user_id) VALUES (?, ?)",
    )
    .bind(quiz_id)
    .bind(user_id)
    .execute(pool.as_ref())
    .await;

    let attempt_id = match attempt_id {
        Err(e) => return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal membuat attempt: {}", e),
            data: None,
            meta: None,
        }),
        Ok(r) => r.last_insert_id() as UInt,
    };

    // Ambil semua soal beserta kunci jawaban
    let questions = sqlx::query_as::<_, (UInt, Str, Option<Str>)>(
        "SELECT id, type, answer_key FROM quiz_question WHERE quiz_id = ?",
    )
    .bind(quiz_id)
    .fetch_all(pool.as_ref())
    .await;

    let questions = match questions {
        Err(e) => return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Error soal: {}", e),
            data: None,
            meta: None,
        }),
        Ok(q) => q,
    };

    // Ambil semua opsi benar
    let correct_options = sqlx::query_as::<_, (UInt, UInt)>(
        r#"SELECT o.id, o.question_id FROM quiz_option o
           JOIN quiz_question q ON q.id = o.question_id
           WHERE q.quiz_id = ? AND o.is_correct = 1"#,
    )
    .bind(quiz_id)
    .fetch_all(pool.as_ref())
    .await;

    let correct_options = match correct_options {
        Err(e) => return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Error opsi: {}", e),
            data: None,
            meta: None,
        }),
        Ok(o) => o,
    };

    let total = questions.len() as f64;
    let mut correct_count: f64 = 0.0;

    // Simpan jawaban dan hitung skor
    for ans in &body.answers {
        let qtype = questions
            .iter()
            .find(|(qid, _, _)| *qid == ans.question_id)
            .map(|(_, t, _)| t.as_str())
            .unwrap_or("");

        let answer_key = questions
            .iter()
            .find(|(qid, _, _)| *qid == ans.question_id)
            .and_then(|(_, _, k)| k.as_deref());

        // Simpan jawaban
        let option_ids_json = ans.option_ids.as_ref().map(|v| {
            serde_json::to_string(v).unwrap_or_else(|_| "[]".into())
        });

        let _ = sqlx::query(
            "INSERT INTO quiz_answer (attempt_id, question_id, answer_text, option_ids)
             VALUES (?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE answer_text = VALUES(answer_text), option_ids = VALUES(option_ids)",
        )
        .bind(attempt_id)
        .bind(ans.question_id)
        .bind(&ans.answer_text)
        .bind(&option_ids_json)
        .execute(pool.as_ref())
        .await;

        // Hitung benar
        match qtype {
            "text" => {
                if let (Some(user_ans), Some(key)) = (&ans.answer_text, answer_key) {
                    if user_ans.trim().to_lowercase() == key.trim().to_lowercase() {
                        correct_count += 1.0;
                    }
                }
            }
            "single" => {
                if let Some(opts) = &ans.option_ids {
                    let user_opt = opts.first().copied().unwrap_or(0);
                    let is_correct = correct_options
                        .iter()
                        .any(|(oid, qid)| *qid == ans.question_id && *oid == user_opt);
                    if is_correct {
                        correct_count += 1.0;
                    }
                }
            }
            "multiple" => {
                if let Some(user_opts) = &ans.option_ids {
                    let correct_for_q: Vec<UInt> = correct_options
                        .iter()
                        .filter(|(_, qid)| *qid == ans.question_id)
                        .map(|(oid, _)| *oid)
                        .collect();

                    let mut user_set: Vec<UInt> = user_opts.clone();
                    user_set.sort();
                    let mut correct_set = correct_for_q.clone();
                    correct_set.sort();

                    if user_set == correct_set {
                        correct_count += 1.0;
                    }
                }
            }
            _ => {}
        }
    }

    let score = if total > 0.0 {
        (correct_count / total * 100.0 * 100.0).round() / 100.0
    } else {
        0.0
    };

    // Update attempt dengan skor dan waktu selesai
    let _ = sqlx::query(
        "UPDATE quiz_attempt SET score = ?, finished_at = NOW() WHERE id = ?",
    )
    .bind(score)
    .bind(attempt_id)
    .execute(pool.as_ref())
    .await;

    Response::Ok().json(ApiResponse {
        success: true,
        message: "Quiz berhasil dikumpulkan".into(),
        data: Some(json!({
            "score": score,
            "correct": correct_count as UInt,
            "total": total as UInt,
        })),
        meta: None,
    })
}

// ─── API: Buat Quiz ───────────────────────────────────────────────────────────

pub async fn api_create_quiz(
    pool: Pool,
    session: Session,
    body: Json<CreateQuizPayload>,
) -> Response {
    let user_id = auth!(session);

    if body.title.trim().is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Judul quiz tidak boleh kosong".into(),
            data: None,
            meta: None,
        });
    }

    if body.questions.is_empty() {
        return Response::BadRequest().json(ApiResponse {
            success: false,
            message: "Quiz harus memiliki minimal 1 soal".into(),
            data: None,
            meta: None,
        });
    }

    // Insert quiz
    let quiz_res = sqlx::query(
        "INSERT INTO quiz (user_id, title, description, is_published) VALUES (?, ?, ?, 1)",
    )
    .bind(user_id)
    .bind(body.title.trim())
    .bind(body.description.as_deref().map(|s| s.trim()))
    .execute(pool.as_ref())
    .await;

    let quiz_id = match quiz_res {
        Err(e) => return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Gagal membuat quiz: {}", e),
            data: None,
            meta: None,
        }),
        Ok(r) => r.last_insert_id() as UInt,
    };

    // Insert soal-soal
    for (i, q) in body.questions.iter().enumerate() {
        let q_res = sqlx::query(
            "INSERT INTO quiz_question (quiz_id, sort_order, question, type, answer_key)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(quiz_id)
        .bind(i as u16)
        .bind(q.question.trim())
        .bind(q.r#type.trim())
        .bind(q.answer_key.as_deref().map(|s| s.trim()))
        .execute(pool.as_ref())
        .await;

        let question_id = match q_res {
            Err(e) => return Response::InternalServerError().json(ApiResponse {
                success: false,
                message: format!("Gagal menyimpan soal: {}", e),
                data: None,
                meta: None,
            }),
            Ok(r) => r.last_insert_id() as UInt,
        };

        // Insert opsi untuk soal single/multiple
        if q.r#type != "text" {
            for opt in &q.options {
                let _ = sqlx::query(
                    "INSERT INTO quiz_option (question_id, label, is_correct) VALUES (?, ?, ?)",
                )
                .bind(question_id)
                .bind(opt.label.trim())
                .bind(opt.is_correct as i8)
                .execute(pool.as_ref())
                .await;
            }
        }
    }

    Response::Ok().json(ApiResponse {
        success: true,
        message: "Quiz berhasil dibuat".into(),
        data: Some(json!({ "quiz_id": quiz_id })),
        meta: None,
    })
}

// ─── API: Delete Quiz ─────────────────────────────────────────────────────────

pub async fn api_delete_quiz(
    pool: Pool,
    session: Session,
    path: Path<(UInt,)>,
) -> Response {
    let user_id = auth!(session);
    let (quiz_id,) = path.into_inner();

    let owner = sqlx::query_as::<_, (UInt,)>(
        "SELECT id FROM quiz WHERE id = ? AND user_id = ?",
    )
    .bind(quiz_id)
    .bind(user_id)
    .fetch_optional(pool.as_ref())
    .await;

    match owner {
        Err(e) => return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Error: {}", e),
            data: None,
            meta: None,
        }),
        Ok(None) => return Response::Forbidden().json(ApiResponse {
            success: false,
            message: "Quiz tidak ditemukan atau bukan milikmu".into(),
            data: None,
            meta: None,
        }),
        Ok(Some(_)) => {}
    }

    let _ = sqlx::query("DELETE FROM quiz WHERE id = ?")
        .bind(quiz_id)
        .execute(pool.as_ref())
        .await;

    Response::Ok().json(ApiResponse {
        success: true,
        message: "Quiz dihapus".into(),
        data: None,
        meta: None,
    })
}

// ─── API: Toggle Publish ──────────────────────────────────────────────────────

pub async fn api_toggle_publish(
    pool: Pool,
    session: Session,
    path: Path<(UInt,)>,
) -> Response {
    let user_id = auth!(session);
    let (quiz_id,) = path.into_inner();

    let row = sqlx::query_as::<_, (UInt, i8)>(
        "SELECT id, is_published FROM quiz WHERE id = ? AND user_id = ?",
    )
    .bind(quiz_id)
    .bind(user_id)
    .fetch_optional(pool.as_ref())
    .await;

    match row {
        Err(e) => return Response::InternalServerError().json(ApiResponse {
            success: false,
            message: format!("Error: {}", e),
            data: None,
            meta: None,
        }),
        Ok(None) => return Response::Forbidden().json(ApiResponse {
            success: false,
            message: "Quiz tidak ditemukan atau bukan milikmu".into(),
            data: None,
            meta: None,
        }),
        Ok(Some((_, current))) => {
            let new_val = if current == 1 { 0i8 } else { 1i8 };
            let _ = sqlx::query("UPDATE quiz SET is_published = ? WHERE id = ?")
                .bind(new_val)
                .bind(quiz_id)
                .execute(pool.as_ref())
                .await;

            Response::Ok().json(ApiResponse {
                success: true,
                message: if new_val == 1 { "Quiz dipublikasikan".into() } else { "Quiz disembunyikan".into() },
                data: Some(json!({ "is_published": new_val == 1 })),
                meta: None,
            })
        }
    }
}
