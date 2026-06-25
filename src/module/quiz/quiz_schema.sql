-- ============================================================
-- MODULE: quiz
-- ============================================================

CREATE TABLE IF NOT EXISTS quiz (
    id          INT UNSIGNED NOT NULL AUTO_INCREMENT,
    user_id     INT UNSIGNED NOT NULL,
    title       VARCHAR(200) NOT NULL,
    description TEXT,
    is_published TINYINT(1) NOT NULL DEFAULT 0,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY idx_quiz_user_id (user_id),
    KEY idx_quiz_published (is_published)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Tipe soal: 'single' (pilihan ganda 1 jawaban), 'multiple' (checklist), 'text' (isian)
CREATE TABLE IF NOT EXISTS quiz_question (
    id          INT UNSIGNED NOT NULL AUTO_INCREMENT,
    quiz_id     INT UNSIGNED NOT NULL,
    sort_order  SMALLINT UNSIGNED NOT NULL DEFAULT 0,
    question    TEXT NOT NULL,
    type        ENUM('single','multiple','text') NOT NULL DEFAULT 'single',
    PRIMARY KEY (id),
    KEY idx_question_quiz_id (quiz_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Pilihan jawaban (hanya untuk tipe single/multiple)
CREATE TABLE IF NOT EXISTS quiz_option (
    id          INT UNSIGNED NOT NULL AUTO_INCREMENT,
    question_id INT UNSIGNED NOT NULL,
    label       VARCHAR(500) NOT NULL,
    is_correct  TINYINT(1) NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    KEY idx_option_question_id (question_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Sesi pengerjaan quiz (satu user satu attempt aktif per quiz)
CREATE TABLE IF NOT EXISTS quiz_attempt (
    id          INT UNSIGNED NOT NULL AUTO_INCREMENT,
    quiz_id     INT UNSIGNED NOT NULL,
    user_id     INT UNSIGNED NOT NULL,
    score       DECIMAL(5,2),
    finished_at DATETIME,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY idx_attempt_quiz_user (quiz_id, user_id),
    KEY idx_attempt_user (user_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Jawaban user per soal
CREATE TABLE IF NOT EXISTS quiz_answer (
    id          INT UNSIGNED NOT NULL AUTO_INCREMENT,
    attempt_id  INT UNSIGNED NOT NULL,
    question_id INT UNSIGNED NOT NULL,
    -- untuk tipe text, jawaban disimpan di answer_text; untuk single/multiple di option_ids JSON
    answer_text TEXT,
    option_ids  JSON,
    PRIMARY KEY (id),
    KEY idx_answer_attempt (attempt_id),
    UNIQUE KEY uq_answer_attempt_question (attempt_id, question_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Jawaban benar untuk soal tipe text (diisi pembuat quiz)
ALTER TABLE quiz_question
    ADD COLUMN answer_key TEXT AFTER type;
