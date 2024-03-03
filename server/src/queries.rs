pub const INSERT_USERS_PUZZLE_STATEMENT: &str = "
            WITH input_data(name, media, email, puzzle_id) AS ( VALUES ($1, $2, $3, unique_short_id())),
            ins_puzzle AS (
                INSERT INTO puzzles (id, name, media) 
                SELECT puzzle_id, name, media FROM input_data ON CONFLICT (id) DO UPDATE SET name=EXCLUDED.name, media=EXCLUDED.media RETURNING id AS puzzle_id, name, media),
                ins_user AS (
                    INSERT INTO users (email) 
                    SELECT email FROM input_data ON CONFLICT (email) DO UPDATE SET email=EXCLUDED.email RETURNING id AS user_id, email),
                    ins_user_puzzle AS (
                        INSERT INTO users_puzzles (user_id, puzzle_id) 
                        SELECT ins_user.user_id, d.puzzle_id FROM input_data as d JOIN ins_user USING (email) ON CONFLICT (user_id, puzzle_id) DO UPDATE SET user_id=EXCLUDED.user_id, puzzle_id=EXCLUDED.puzzle_id 
                        RETURNING user_id, puzzle_id) 
                        SELECT user_id, puzzle_id, name, media, email FROM ins_user_puzzle JOIN ins_user USING(user_id) JOIN ins_puzzle USING (puzzle_id);
        ";

pub const UPDATE_USERS_PUZZLE_STATEMENT: &str = "
            WITH input_data(name, media, email, puzzle_id) AS ( VALUES ($1, $2, $3, $4)),
            ins_puzzle AS (
                INSERT INTO puzzles (id, name, media) 
                SELECT puzzle_id, name, media FROM input_data ON CONFLICT (id) DO UPDATE SET name=EXCLUDED.name, media=EXCLUDED.media RETURNING id AS puzzle_id, name, media),
                ins_user AS (
                    INSERT INTO users (email) 
                    SELECT email FROM input_data ON CONFLICT (email) DO UPDATE SET email=EXCLUDED.email RETURNING id AS user_id, email),
                    ins_user_puzzle AS (
                        INSERT INTO users_puzzles (user_id, puzzle_id) 
                        SELECT ins_user.user_id, d.puzzle_id FROM input_data as d JOIN ins_user USING (email) ON CONFLICT (user_id, puzzle_id) DO UPDATE SET user_id=EXCLUDED.user_id, puzzle_id=EXCLUDED.puzzle_id 
                        RETURNING user_id, puzzle_id) 
                        SELECT user_id, puzzle_id, name, media, email FROM ins_user_puzzle JOIN ins_user USING(user_id) JOIN ins_puzzle USING (puzzle_id);
        ";
