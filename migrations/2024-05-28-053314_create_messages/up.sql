CREATE TABLE IF NOT EXISTS messages
(
    id         TEXT PRIMARY KEY,
    nonce      TEXT NOT NULL,
    ciphertext TEXT NOT NULL
);