CREATE TABLE IF NOT EXISTS post (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    seo_title VARCHAR(255) NOT NULL,
    slug VARCHAR(255) UNIQUE NOT NULL,
    photo_url VARCHAR(255) NOT NULL,
    tags VARCHAR(255) NOT NULL,
    summary TEXT NOT NULL,
    content TEXT NOT NULL,
    date INT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_post_slug ON post (slug);
