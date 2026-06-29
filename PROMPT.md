Buatkan halaman baru untuk aplikasi Rust menggunakan Actix Web dengan struktur berikut:

## Stack
- Framework: Actix Web
- Database: SQLx MySQL
- Session: actix-session
- Password Hash: bcrypt
- Routing dilakukan di main.rs menggunakan .route()
- Menggunakan sistem modular
- Router berada di src/module/{nama_module}/mod.rs 
- Handler berada di src/module/{nama_module}/handler.rs
- HTML berada di src/module/{nama_module}/page_{nama_halaman}.html

## Alias yang WAJIB digunakan
```rust
use crate::web::{Data, Pool, Session, Cookie, Request, Response, ApiResponse};
use crate::web::from::{Path, Json, Form, Multipart, Socket, Sse};
use crate::web::data::{Int, UInt, Uuid, String, Date};
```

Referensi tipe:
- `Data`     = `actix_web::web::Data`
- `Pool`     = `Data<MySqlPool>`
- `Request`  = `actix_web::HttpRequest`
- `Response` = `actix_web::HttpResponse`
- `Session`  = `actix_session::Session`
- `Cookie`   = `actix_web::cookie::Cookie`
- `Multipart`= `actix_multipart::Multipart`
- `Int`      = `i32`
- `UInt`     = `u32`
- `Uuid`     = `uuid::Uuid`
- `String`   = `std::string::String`
- `Date`     = `chrono::DateTime<Utc>`


## Aturan Router
- Terdapat `pub mod handler;`
- Tambahkan fungsi `open_routes(cfg: &mut ServiceConfig)` untuk daftar route tanpa auth
- Tambahkan fungsi `gate_routes(cfg: &mut ServiceConfig)` untuk daftar route dengan auth, gunakan hanya untuk API, jangan gunakan untuk halaman
- Gate routes sudah memiliki prefix atau scope `/gate` di main.rs, setiap diakses harus memiliki awalan `/gate`
- Gunakan `cfg.service()` dan `scope("/url_utama"))` untuk menambahkan route
- Jangan tambahkan route diluar url_utama


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
- Gunakan `pub struct` untuk membuat struct baru
- Handler tidak boleh mengandung string HTML
- Jika ada halaman yang membutuhkan auth, redirect ke `/auth/login`

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
Macro ini akan me-return `user_id` bertipe `UInt` jika user atau `None` jika bukan user.
Jangan gunakan `session.get::<UInt>("user_id")` manual jika halaman berstatus Login Required — gunakan `auth!(session)`.


## Aturan HTML
- HTML dan Petite-Vue, tidak ada kode Rust di dalamnya
- Tailwind CSS (utility-first, tanpa custom CSS)
- Petite-Vue untuk state lokal dan interaktivitas, kosongkan v-scope, taruh logika di createApp()
- Fetch API untuk komunikasi dengan backend
- Mobile friendly
- Gunakan source `https://cdn.tailwindcss.com` sebagai Tailwind
- Gunakan source `https://unpkg.com/petite-vue@0.4.1/dist/petite-vue.iife.js` sebagai Petite-Vue
- Jagan gunakan `init` dan `defer`
- Jika menggunakan icon usahakan svg dari iconify atau heroicons, jangan gunakan CDN
- Mount tanpa selector via v-scope, panggil method dari object asli.
- Gunakan satu state untuk setiap halaman (`const state = { ... }`) dan satu mount point (`PetiteVue.createApp(state).mount('#app')`).
- Panggil method fetch dari state langsung setelah dibuat (`state.fetchData()`) jika diperlukan.


## Desain
- Modern, minimalis, profesional
- Responsive (mobile-first)
- Tailwind utility-first
- Tidak ada framework CSS tambahan


## Aturan Database
- Untuk id gunakan UNSIGNED INTEGER
- Gunakan struktur yang efisien dan cepat untuk penggunaan data besar
- Tebel `users` berisi kolom `id, username, fullname, password, last_login, created_at`


## Output yang harus dihasilkan
Berikan output dalam urutan berikut:
1. **SQL** — struktur tabel jika diperlukan
2. **Route** — file lengkap mod.rs
3. **Handler** — file lengkap handler.rs
4. **HTML** — file lengkap nama.html
5. **Endpoint tambahan** — handler tambahan jika ada (misal: search, pagination, delete)
6. **Alur halaman** — penjelasan singkat alur dari request pertama hingga render

---

## Spesifikasi Module / Halaman
**Buatkan:** Module / Halaman
**Nama Module/Halaman:** [isi]
**URL Utama:** [isi]
**Status:** Public / Login Required
**Warna Dominan:** [isi]
**Keterangan:** [isi — jelaskan fungsi modul atau halaman, data apa yang ditampilkan, aksi apa yang bisa dilakukan]