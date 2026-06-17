-- ============================================================
-- TABEL: forum_tags
-- ============================================================
CREATE TABLE forum_tags (
    id         INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    name       VARCHAR(100) NOT NULL,
    slug       VARCHAR(100) NOT NULL UNIQUE,
    color      VARCHAR(7) NOT NULL DEFAULT '#f97316',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================
-- TABEL: forum_threads
-- ============================================================
CREATE TABLE forum_threads (
    id             INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    user_id        INT NOT NULL,
    title          VARCHAR(255) NOT NULL,
    content        TEXT NOT NULL,
    tag_id         INT DEFAULT NULL,
    access_type    ENUM('public','user','password') NOT NULL DEFAULT 'public',
    password_hash  VARCHAR(255) DEFAULT NULL,
    created_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (tag_id)  REFERENCES forum_tags(id) ON DELETE SET NULL
);

-- ============================================================
-- TABEL: forum_replies
-- ============================================================
CREATE TABLE forum_replies (
    id         INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    thread_id  INT NOT NULL,
    user_id    INT NOT NULL,
    content    TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (thread_id) REFERENCES forum_threads(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id)   REFERENCES users(id)
);

-- ============================================================
-- INDEX
-- ============================================================
CREATE INDEX idx_ft_tag        ON forum_threads(tag_id);
CREATE INDEX idx_ft_user       ON forum_threads(user_id);
CREATE INDEX idx_ft_created    ON forum_threads(created_at);
CREATE INDEX idx_fr_thread     ON forum_replies(thread_id);
CREATE INDEX idx_fr_user       ON forum_replies(user_id);

-- ============================================================
-- DATA AWAL: Tag default
-- ============================================================
INSERT INTO forum_tags (name, slug, color) VALUES
('Diskusi Umum', 'diskusi-umum',  '#f97316'),
('Pertanyaan',    'pertanyaan',     '#3b82f6'),
('Tutorial',      'tutorial',       '#10b981'),
('Bug Report',    'bug-report',     '#ef4444'),
('Off-Topic',     'off-topic',      '#8b5cf6');