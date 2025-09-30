use ncurses::*;
use std::time::{Duration, Instant};

// --- Game Constants ---
const MAX_PLAYER_X: u16 = 38;
const MAX_PLAYER_Y: u16 = 20;
const ALIEN_ROWS: usize = 2;
const ALIEN_COLS: usize = 6;
const HORIZONTAL_SPACING: u16 = 5;
const VERTICAL_SPACING: u16 = 4;
const MAX_SHOTS: usize = 10;
const ALIEN_FIRE_INTERVAL: Duration = Duration::from_millis(750);
const INITIAL_LIVES: u8 = 3;

// --- Color Pair Definitions ---
const COLOR_UI: i16 = 1;
const COLOR_PLAYER: i16 = 2;
const COLOR_SHOT: i16 = 3;
const COLOR_ALIEN: i16 = 4;
const COLOR_GAMEOVER: i16 = 5;
const COLOR_ALIEN_SHOT: i16 = 6;

// --- Key Code Constants for Match Patterns ---
const KEY_Q: i32 = 'q' as i32;
const KEY_A: i32 = 'a' as i32;
const KEY_D: i32 = 'd' as i32;
const KEY_SPACE: i32 = ' ' as i32;


// --- Data Structures ---
struct Player {
    x: u16,
    y: u16,
}

struct Alien {
    x: u16,
    y: u16,
}

struct Shot {
    x: u16,
    y: u16,
}

struct AlienShot {
    x: u16,
    y: u16,
}

enum AlienDirection {
    Left,
    Right,
}

struct GameState {
    player: Player,
    shots: Vec<Shot>,
    aliens: Vec<Alien>,
    alien_shots: Vec<AlienShot>,
    last_alien_shot: Instant,
    alien_direction: AlienDirection,
    score: u32,
    lives: u8,
    game_over: bool,
}

// --- Sprites ---
const ALIEN_SPRITE: [&'static str; 2] = ["<O>", "/-\\" ];
const PLAYER_SPRITE: [&'static str; 2] = ["/A\\", "===" ];

// --- Helper Functions ---
fn spawn_new_wave(state: &mut GameState) {
    // Clear any remaining shots from the previous level
    state.shots.clear();
    state.alien_shots.clear();

    // Repopulate aliens
    let mut aliens = Vec::new();
    for row in 0..ALIEN_ROWS {
        for col in 0..ALIEN_COLS {
            aliens.push(Alien {
                x: (col as u16) * HORIZONTAL_SPACING + 2,
                y: (row as u16) * VERTICAL_SPACING + 2,
            });
        }
    }
    state.aliens = aliens;
}


// --- Game Rendering (ncurses) ---

fn render(state: &GameState) {
    // Erase the screen instead of clearing it to prevent flicker
    erase();

    // Render UI (Score, Lives, and instructions)
    attron(COLOR_PAIR(COLOR_UI));
    let ui_text = format!("Score: {} | Lives: {} | Press 'q' to quit", state.score, state.lives);
    mvaddstr(0, 0, &ui_text);
    attroff(COLOR_PAIR(COLOR_UI));

    // Render Player
    if !state.game_over {
        attron(COLOR_PAIR(COLOR_PLAYER));
        for (i, line) in PLAYER_SPRITE.iter().enumerate() {
            mvaddstr((state.player.y + i as u16) as i32, state.player.x as i32, line);
        }
        attroff(COLOR_PAIR(COLOR_PLAYER));
    }

    // Render Shots
    attron(COLOR_PAIR(COLOR_SHOT));
    for shot in &state.shots {
        mvaddstr(shot.y as i32, shot.x as i32, "|");
    }
    attroff(COLOR_PAIR(COLOR_SHOT));

    // Render Alien Shots
    attron(COLOR_PAIR(COLOR_ALIEN_SHOT));
    for shot in &state.alien_shots {
        mvaddstr(shot.y as i32, shot.x as i32, "v");
    }
    attroff(COLOR_PAIR(COLOR_ALIEN_SHOT));

    // Render Aliens
    attron(COLOR_PAIR(COLOR_ALIEN));
    for alien in &state.aliens {
        for (i, line) in ALIEN_SPRITE.iter().enumerate() {
            mvaddstr((alien.y + i as u16) as i32, alien.x as i32, line);
        }
    }
    attroff(COLOR_PAIR(COLOR_ALIEN));
    
    // Render Game Over message
    if state.game_over {
        attron(COLOR_PAIR(COLOR_GAMEOVER));
        mvaddstr((MAX_PLAYER_Y / 2) as i32, 15, "GAME OVER!");
        mvaddstr(((MAX_PLAYER_Y / 2) + 1) as i32, 10, &format!("Final Score: {}", state.score));
        mvaddstr(((MAX_PLAYER_Y / 2) + 2) as i32, 8, "Press 'q' to exit.");
        attroff(COLOR_PAIR(COLOR_GAMEOVER));
    }
    
    // Refresh the screen to show changes
    refresh();
}

// --- Game Logic ---

fn update_state(state: &mut GameState) {
    if state.game_over {
        return;
    }

    // --- Player Logic ---
    // Update shot positions and remove off-screen shots
    if !state.shots.is_empty() {
        for shot in &mut state.shots {
            shot.y -= 1;
        }
        state.shots.retain(|shot| shot.y > 1);
    }

    // --- Alien Logic ---
    // Update alien shot positions
    if !state.alien_shots.is_empty() {
        for shot in &mut state.alien_shots {
            shot.y += 1;
        }
        // Remove off-screen alien shots
        state.alien_shots.retain(|shot| shot.y < MAX_PLAYER_Y + 2);
    }

    // --- Collision Detection ---
    // Check if alien shot hits player
    let mut player_hit = false;
    state.alien_shots.retain(|shot| {
        let hit = shot.x >= state.player.x
            && shot.x < state.player.x + 3
            && shot.y >= state.player.y
            && shot.y < state.player.y + 2;
        if hit {
            player_hit = true;
        }
        !hit // Keep shot if it didn't hit
    });

    if player_hit {
        state.lives -= 1;
        state.player.x = MAX_PLAYER_X / 2; // Reset player position
        if state.lives == 0 {
            state.game_over = true;
            return;
        }
    }

    // Collision detection for player shots hitting aliens
    if !state.shots.is_empty() && !state.aliens.is_empty() {
        let mut aliens_alive: Vec<bool> = vec![true; state.aliens.len()];
        let mut shots_to_keep: Vec<bool> = vec![true; state.shots.len()];

        for (i, shot) in state.shots.iter().enumerate() {
            for (j, alien) in state.aliens.iter().enumerate() {
                if aliens_alive[j] { // Only check against live aliens
                    if shot.x >= alien.x
                        && shot.x < alien.x + 3
                        && shot.y >= alien.y
                        && shot.y < alien.y + 2
                    {
                        aliens_alive[j] = false;
                        shots_to_keep[i] = false;
                        state.score += 10;
                        break; // Shot is used up, move to next shot
                    }
                }
            }
        }
        
        // Filter out dead aliens
        let mut updated_aliens = Vec::new();
        for (i, alien) in state.aliens.drain(..).enumerate() {
            if aliens_alive[i] {
                updated_aliens.push(alien);
            }
        }
        state.aliens = updated_aliens;

        // Filter out used shots
        let mut updated_shots = Vec::new();
        for (i, shot) in state.shots.drain(..).enumerate() {
            if shots_to_keep[i] {
                updated_shots.push(shot);
            }
        }
        state.shots = updated_shots;
    }
    
    // --- Alien Firing Logic ---
    if Instant::now().duration_since(state.last_alien_shot) > ALIEN_FIRE_INTERVAL && !state.aliens.is_empty() {
        let mut potential_shooters: Vec<&Alien> = Vec::new();
        // Find aliens in the front rank (no other aliens below them in the same column)
        for alien_a in &state.aliens {
            let mut is_front_rank = true;
            for alien_b in &state.aliens {
                if (alien_b.x..alien_b.x + 3).contains(&alien_a.x) && alien_a.y < alien_b.y {
                    is_front_rank = false;
                    break;
                }
            }
            if is_front_rank {
                potential_shooters.push(alien_a);
            }
        }

        if !potential_shooters.is_empty() {
            // "Randomly" pick a shooter
            let now_nanos = Instant::now().duration_since(state.last_alien_shot).as_nanos();
            let shooter = potential_shooters[(now_nanos as usize) % potential_shooters.len()];
            state.alien_shots.push(AlienShot { x: shooter.x + 1, y: shooter.y + 2 });
            state.last_alien_shot = Instant::now();
        }
    }

    // --- Level Progression ---
    if state.aliens.is_empty() {
        spawn_new_wave(state);
        return;
    }

    // Update alien positions
    let mut wall_hit = false;
    for alien in &state.aliens {
        match state.alien_direction {
            AlienDirection::Left => {
                if alien.x == 0 {
                    wall_hit = true;
                    break;
                }
            }
            AlienDirection::Right => {
                if alien.x >= MAX_PLAYER_X - 1 {
                    wall_hit = true;
                    break;
                }
            }
        }
    }

    if wall_hit {
        state.alien_direction = match state.alien_direction {
            AlienDirection::Left => AlienDirection::Right,
            AlienDirection::Right => AlienDirection::Left,
        };
        for alien in &mut state.aliens {
            alien.y += 1;
             if alien.y + 1 >= state.player.y {
                state.game_over = true;
                return;
            }
        }
    } else {
        for alien in &mut state.aliens {
            match state.alien_direction {
                AlienDirection::Left => alien.x -= 1,
                AlienDirection::Right => alien.x += 1,
            }
        }
    }
}

// --- Main Game Loop ---

fn main() {
    // Setup ncurses
    initscr();
    start_color();
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    timeout(100); // Set non-blocking input
    keypad(stdscr(), true); // Enable keypad keys
    leaveok(stdscr(), true); // Optimization to reduce cursor movement

    // Initialize color pairs
    init_pair(COLOR_UI, COLOR_YELLOW, COLOR_BLACK);
    init_pair(COLOR_PLAYER, COLOR_CYAN, COLOR_BLACK);
    init_pair(COLOR_SHOT, COLOR_RED, COLOR_BLACK);
    init_pair(COLOR_ALIEN, COLOR_GREEN, COLOR_BLACK);
    init_pair(COLOR_GAMEOVER, COLOR_RED, COLOR_BLACK);
    init_pair(COLOR_ALIEN_SHOT, COLOR_MAGENTA, COLOR_BLACK);

    // Game state initialization
    let mut state = GameState {
        player: Player {
            x: MAX_PLAYER_X / 2,
            y: MAX_PLAYER_Y,
        },
        shots: Vec::new(),
        alien_shots: Vec::new(),
        last_alien_shot: Instant::now(),
        aliens: Vec::new(), // Start with an empty vec, spawn_new_wave will populate it
        alien_direction: AlienDirection::Right,
        score: 0,
        lives: INITIAL_LIVES,
        game_over: false,
    };
    
    // Spawn the first wave of aliens
    spawn_new_wave(&mut state);

    let mut last_update = Instant::now();
    let update_interval = Duration::from_millis(200);

    'gameloop: loop {
        // Update game state at a fixed interval
        if last_update.elapsed() >= update_interval {
            update_state(&mut state);
            last_update = Instant::now();
        }

        // Render the current state
        render(&state);

        // Handle user input
        match getch() {
            // Quit
            KEY_Q => break 'gameloop,
            // Movement
            KEY_A | KEY_LEFT => {
                if state.player.x > 0 && !state.game_over {
                    state.player.x -= 1;
                }
            }
            KEY_D | KEY_RIGHT => {
                // Adjust boundary for 3-char wide sprite
                if state.player.x < MAX_PLAYER_X - 2 && !state.game_over {
                    state.player.x += 1;
                }
            }
            // Shooting
            KEY_SPACE => {
                if state.shots.len() < MAX_SHOTS && !state.game_over {
                    // Fire from the center of the vessel
                    let new_shot = Shot { x: state.player.x + 1, y: state.player.y - 1 };
                    state.shots.push(new_shot);
                }
            }
            _ => {}
        }
    }

    // Cleanup ncurses
    endwin();
}

