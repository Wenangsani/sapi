--
-- Table structure for table `users`
--

CREATE TABLE users (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,

    username VARCHAR(100) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    fullname VARCHAR(100) NOT NULL DEFAULT '',
    role ENUM('admin','user') NOT NULL DEFAULT 'user',

    last_login DATETIME NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    INDEX idx_role (role)
);

-- Tambahan untuk superadmin module:

CREATE TABLE uploaded_files (
    id         INT UNSIGNED NOT NULL AUTO_INCREMENT,
    filename   VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    mime_type  VARCHAR(127) NOT NULL,
    file_size  BIGINT UNSIGNED NOT NULL DEFAULT 0,
    uploaded_by INT UNSIGNED NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY idx_uploaded_by (uploaded_by),
    KEY idx_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE app_stats_snapshot (
    id         INT UNSIGNED NOT NULL AUTO_INCREMENT,
    stat_key   VARCHAR(64) NOT NULL,
    stat_value BIGINT NOT NULL DEFAULT 0,
    recorded_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY idx_key_time (stat_key, recorded_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;