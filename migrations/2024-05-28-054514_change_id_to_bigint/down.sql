-- Удаление новой таблицы
DROP TABLE IF EXISTS messages;

-- Восстановление старой таблицы с UUID
CREATE TABLE messages
(
    id         UUID PRIMARY KEY,
    nonce      TEXT NOT NULL,
    ciphertext TEXT NOT NULL
);
