CREATE TABLE users (
    discord_id VARCHAR(20) PRIMARY KEY,
    username VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE smoking_types (
    id SERIAL PRIMARY KEY,
    type_name VARCHAR(50) NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO smoking_types (type_name, description) 
VALUES 
    ('traditional', '紙タバコ'),
    ('iqos', 'IQOS');

CREATE TABLE smoking_logs (
    id SERIAL PRIMARY KEY,
    discord_id VARCHAR(20) REFERENCES users(discord_id),
    smoking_type_id INTEGER REFERENCES smoking_types(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    smoked_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_smoking_logs_discord_id ON smoking_logs(discord_id);
CREATE INDEX idx_smoking_logs_smoked_at ON smoking_logs(smoked_at);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_smoking_logs_updated_at
    BEFORE UPDATE ON smoking_logs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE VIEW daily_smoking_summary AS
SELECT 
    sl.discord_id,
    u.username,
    DATE(sl.smoked_at) as smoke_date,
    st.type_name,
    SUM(sl.quantity) as total_quantity
FROM smoking_logs sl
JOIN users u ON sl.discord_id = u.discord_id
JOIN smoking_types st ON sl.smoking_type_id = st.id
GROUP BY 
    sl.discord_id,
    u.username,
    DATE(sl.smoked_at),
    st.type_name;

