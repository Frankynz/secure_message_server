
# Secure Message Server

Secure Message Server позволяет безопасно передавать сообщения с ограниченным количеством просмотров.

## Требования

- Rust (установить можно с помощью [rustup](https://rustup.rs/))
- PostgreSQL (установите и запустите PostgreSQL сервер)
- Diesel CLI (установите с помощью команды `cargo install diesel_cli --no-default-features --features postgres`)
- Docker и Docker Compose (для использования Docker)

## Сборка и запуск проекта

### 1. Клонирование репозитория

```sh
git clone https://github.com/yourusername/secure_message_server.git
cd secure_message_server
```

### 2. Создание и настройка `.env` файла

В корне проекта создайте файл `.env` со следующим содержимым:

```env
DATABASE_URL=postgres://username:password@localhost/dbname
SERVER_ADDRESS=127.0.0.1
SERVER_PORT=8080
```

Замените `username`, `password`, и `dbname` на ваши данные для подключения к PostgreSQL.

### 3. Установка зависимостей

В корне проекта выполните команду:

```sh
cargo build
```

### 4. Применение миграций

Убедитесь, что ваш PostgreSQL сервер запущен, затем выполните:

```sh
diesel migration run
```

### 5. Запуск сервера

Выполните команду:

```sh
cargo run
```

Сервер будет запущен и будет слушать на адресе и порту, указанном в `.env` файле.

### 6. Тестирование API

Вы можете использовать `curl` или любой другой инструмент для тестирования HTTP запросов, чтобы протестировать API.

#### Отправка сообщения:

```sh
curl --location 'http://localhost:8080/messages' --header 'Content-Type: application/json' --data '{
    "content": "Hello, world!",
    "views_left": 2
}'
```

Если запрос успешен, вы получите JSON ответ с URL для получения сообщения.

#### Получение сообщения:

Используйте URL, который вы получили в ответе на предыдущий запрос. Например:

```sh
curl --location 'http://localhost:8080/messages/1'
```

Если сообщение доступно, вы получите его содержимое. Если сообщение уже было просмотрено необходимое количество раз, вы получите ответ "Message not found" с кодом 404.

## Использование Docker

### Docker Compose

В проекте также есть файл `DockerCompose.yml`, который находится по пути `/docker/DockerCompose.yml`.

### Запуск с помощью Docker Compose

1. Перейдите в каталог с Docker Compose файлом:

   ```sh
   cd docker
   ```

2. Запустите контейнеры:

   ```sh
   docker-compose up --build
   ```

   Это создаст и запустит контейнеры для сервера и базы данных PostgreSQL.

## Описание полей

- `content`: Текст сообщения, которое вы хотите отправить.
- `views_left`: Количество раз, которое сообщение может быть просмотрено. Если указано `-1`, сообщение никогда не будет удалено. По умолчанию значение `1`.

## Пример использования

### Отправка сообщения с одним просмотром (по умолчанию):

```sh
curl --location 'http://localhost:8080/messages' --header 'Content-Type: application/json' --data '{
    "content": "Hello, world!"
}'
```

### Отправка сообщения с двумя просмотрами:

```sh
curl --location 'http://localhost:8080/messages' --header 'Content-Type: application/json' --data '{
    "content": "Hello, world!",
    "views_left": 2
}'
```

### Получение сообщения:

```sh
curl --location 'http://localhost:8080/messages/1'
```

Если сообщение доступно, вы получите его содержимое. Если сообщение уже было просмотрено необходимое количество раз, вы получите ответ "Message not found" с кодом 404.

## Структура проекта

- `src/main.rs`: Главный файл проекта, содержащий настройки сервера и обработчики маршрутов.
- `src/db.rs`: Файл для настройки соединения с базой данных.
- `src/storage.rs`: Файл для работы с базой данных (сохранение, получение и удаление сообщений).
- `src/schema.rs`: Схема базы данных, сгенерированная Diesel.
- `migrations/`: Каталог с миграциями для базы данных.
- `docker/DockerCompose.yml`: Файл для настройки Docker Compose.