Buatkan halaman baru untuk aplikasi Rust menggunakan Actix Web dengan struktur berikut:

## Stack

- Framework: Actix Web
- Database: SQLx MySQL
- Session: actix-session
- Password Hash: bcrypt
- Routing dilakukan di main.rs menggunakan .route()
- Handler berada di folder src/handler/
- File HTML berada di folder src/page/

## Alias yang WAJIB digunakan

```rust
use crate::web::{Pool, Session, Cookie, Request, Response, ApiResponse};
use crate::web::from::{Data, Path, Json, Form};
use crate::web::data::{Int, String, Date};
```

Referensi tipe:
- `Pool`     = `Data<MySqlPool>`
- `Request`  = `HttpRequest`
- `Response` = `HttpResponse`
- `Session`  = `actix_session::Session`
- `Cookie`   = `actix_web::cookie::Cookie`
- `Int`      = `i32`
- `String`   = `std::string::String`
- `Date`     = `chrono::DateTime<Utc>`

## Aturan Handler

- Return type wajib `Response`
- Gunakan `Pool` untuk query database
- Gunakan `Session` untuk autentikasi
- Gunakan `Json<T>` atau `Form<T>` untuk input body
- Gunakan `Path<T>` untuk url path
- Gunakan `sqlx::query()` atau `sqlx::query_as::<_, T>()`
- Tangani semua error database — tidak boleh ada `.unwrap()` pada operasi fallible
- Boleh `.unwrap()` hanya pada nilai yang dijamin tidak None/Err (contoh: nilai literal)
- Dilarang menggunakan `anyhow`, `Box<dyn Error>`, atau `unwrap()` sembarangan
- Handler tidak boleh mengandung string HTML

Format response error:
```rust
return Response::BadRequest().json(ApiResponse {
    success: false,
    message: "pesan_error".into(),
    data: None,
    meta: None,
});
```

Format response sukses:
```rust
return Response::Ok().json(ApiResponse {
    success: true,
    message: "pesan_sukses".into(),
    data: Some(json!({ ... })),
    meta: None,
});
```

## Macro yang Tersedia

Gunakan macro `auth!` untuk guard autentikasi pada halaman Login Required:

```rust
let user_id = auth!(session);
```

Macro ini akan otomatis return `401 Unauthorized` jika session tidak valid. Setelah baris ini, `user_id` bertipe `Int` dan dijamin terisi.
Jangan gunakan `session.get::<Int>("user_id")` manual jika halaman berstatus Login Required — gunakan `auth!(session)`.


## Aturan HTML

- Berada di `src/page/`
- HTML dan Petite-Vue, tidak ada kode Rust di dalamnya
- Tailwind CSS (utility-first, tanpa custom CSS kecuali benar-benar perlu)
- Petite-Vue untuk state lokal dan interaktivitas
- Fetch API untuk komunikasi dengan backend
- Mobile friendly

## Desain

- Modern, minimalis, profesional
- Responsive (mobile-first)
- Tailwind utility-first
- Tidak ada framework CSS tambahan

## Output yang harus dihasilkan

Berikan output dalam urutan berikut:
1. **SQL** — struktur tabel jika diperlukan
2. **Route** — baris yang ditambahkan di main.rs
3. **Handler** — file lengkap di src/handler/nama.rs
4. **HTML** — file lengkap di src/page/nama.html
5. **Endpoint tambahan** — handler tambahan jika ada (misal: search, pagination, delete)
6. **Alur halaman** — penjelasan singkat alur dari request pertama hingga render

---

## Spesifikasi Halaman

**Nama halaman:** [isi]
**URL:** [isi]
**Status:** Public / Login Required
**Keterangan:** [isi — jelaskan fungsi halaman, data apa yang ditampilkan, aksi apa yang bisa dilakukan]