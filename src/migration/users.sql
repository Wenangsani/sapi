--
-- Table structure for table `users`
--

CREATE TABLE users (
    id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,

    username VARCHAR(100) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    fullname VARCHAR(100) NOT NULL DEFAULT '',
    role ENUM('admin','user') NOT NULL DEFAULT 'user',

    last_login TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    INDEX idx_role (role),
    INDEX idx_active (is_active)
);