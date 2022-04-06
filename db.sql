CREATE TABLE IF NOT EXISTS app_users
(
    id         SERIAL PRIMARY KEY NOT NULL,
    first_name VARCHAR(255),
    last_name  VARCHAR(255),
    email      VARCHAR(255)       NOT NULL UNIQUE,
    pwd        VARCHAR(255)       NOT NULL,
    created_at timestamp with time zone DEFAULT (now() at time zone 'utc')
);

CREATE TABLE IF NOT EXISTS projects
(
    id          SERIAL PRIMARY KEY NOT NULL,
    title       VARCHAR(255)       NOT NULL,
    description TEXT,
    created_at  timestamp with time zone DEFAULT (now() at time zone 'utc')
);

CREATE TABLE IF NOT EXISTS tasks
(
    id          SERIAL PRIMARY KEY NOT NULL,
    title       VARCHAR(255)       NOT NULL,
    description TEXT,
    user_id     int4               NOT NULL,
    project_id  int4               NOT NULL,
    created_at  timestamp with time zone DEFAULT (now() at time zone 'utc'),
    FOREIGN KEY (user_id) references app_users (id),
    FOREIGN KEY (project_id) references projects (id)
);

CREATE TABLE IF NOT EXISTS users_projects
(
    user_id    int4 NOT NULL,
    project_id int4 NOT NULL,
    PRIMARY KEY (user_id, project_id),
    FOREIGN KEY (user_id) REFERENCES app_users (id),
    FOREIGN KEY (project_id) REFERENCES projects (id)
);

