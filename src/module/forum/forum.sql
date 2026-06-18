-- =========================================================
-- FORUM MODULE - DATABASE SCHEMA
-- =========================================================

-- Tabel thread utama
CREATE TABLE forum_threads (
    id              INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user_id         INT UNSIGNED NOT NULL,
    title           VARCHAR(255) NOT NULL,
    content         TEXT NOT NULL,
    access_type     ENUM('public', 'user', 'password') NOT NULL DEFAULT 'public',
    access_password VARCHAR(255) NULL, -- bcrypt hash, hanya diisi jika access_type = 'password'
    view_count      INT UNSIGNED NOT NULL DEFAULT 0,
    reply_count     INT UNSIGNED NOT NULL DEFAULT 0,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    INDEX idx_forum_threads_user (user_id),
    INDEX idx_forum_threads_created (created_at),
    INDEX idx_forum_threads_access (access_type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Tabel tag master
CREATE TABLE forum_tags (
    id          INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    name        VARCHAR(50) NOT NULL,
    slug        VARCHAR(50) NOT NULL,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    UNIQUE KEY uq_forum_tags_slug (slug)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Pivot thread <-> tag (many-to-many)
CREATE TABLE forum_thread_tags (
    thread_id   INT UNSIGNED NOT NULL,
    tag_id      INT UNSIGNED NOT NULL,

    PRIMARY KEY (thread_id, tag_id),
    INDEX idx_forum_thread_tags_tag (tag_id),

    CONSTRAINT fk_forum_thread_tags_thread FOREIGN KEY (thread_id) REFERENCES forum_threads(id) ON DELETE CASCADE,
    CONSTRAINT fk_forum_thread_tags_tag FOREIGN KEY (tag_id) REFERENCES forum_tags(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Tabel reply
CREATE TABLE forum_replies (
    id          INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    thread_id   INT UNSIGNED NOT NULL,
    user_id     INT UNSIGNED NOT NULL,
    content     TEXT NOT NULL,
    created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    INDEX idx_forum_replies_thread (thread_id, created_at),
    INDEX idx_forum_replies_user (user_id),

    CONSTRAINT fk_forum_replies_thread FOREIGN KEY (thread_id) REFERENCES forum_threads(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- Tabel verifikasi akses thread berpassword (agar user tidak diminta password berulang kali per-session)
CREATE TABLE forum_thread_unlocks (
    thread_id   INT UNSIGNED NOT NULL,
    user_id     INT UNSIGNED NOT NULL,
    unlocked_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (thread_id, user_id),

    CONSTRAINT fk_forum_thread_unlocks_thread FOREIGN KEY (thread_id) REFERENCES forum_threads(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;