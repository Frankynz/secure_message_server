-- Удаление старой таблицы
DROP TABLE IF EXISTS messages;

-- Создание новой таблицы с bigint serial
CREATE TABLE messages
(
    id         BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    nonce      TEXT NOT NULL,
    ciphertext TEXT NOT NULL
);
