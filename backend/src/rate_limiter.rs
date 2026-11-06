use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

// Структура для хранения информации о попытках
struct AttemptInfo {
    attempts: Vec<Instant>,
    blocked_until: Option<Instant>,
}

// Глобальный rate limiter
lazy_static::lazy_static! {
    static ref RATE_LIMITER: Mutex<HashMap<IpAddr, AttemptInfo>> = Mutex::new(HashMap::new());
}

// Настройки rate limiting
const MAX_ATTEMPTS: usize = 5;
const WINDOW_DURATION: Duration = Duration::from_secs(5 * 60); // 5 минут
const BLOCK_DURATION: Duration = Duration::from_secs(5 * 60);  // 5 минут блокировки

pub fn check_rate_limit(ip: IpAddr) -> Result<(), String> {
    let mut limiter = RATE_LIMITER.lock();
    let now = Instant::now();

    // Получаем или создаём запись для этого IP
    let info = limiter.entry(ip).or_insert_with(|| AttemptInfo {
        attempts: Vec::new(),
        blocked_until: None,
    });

    // Проверяем, заблокирован ли IP
    if let Some(blocked_until) = info.blocked_until {
        if now < blocked_until {
            let remaining = blocked_until.duration_since(now);
            let minutes = remaining.as_secs() / 60;
            let seconds = remaining.as_secs() % 60;
            return Err(format!(
                "Слишком много попыток. Подождите {} мин {} сек",
                minutes, seconds
            ));
        } else {
            // Блокировка истекла, сбрасываем
            info.blocked_until = None;
            info.attempts.clear();
        }
    }

    // Удаляем старые попытки (старше 5 минут)
    info.attempts.retain(|&t| now.duration_since(t) < WINDOW_DURATION);

    // Проверяем количество попыток
    if info.attempts.len() >= MAX_ATTEMPTS {
        // Блокируем IP
        info.blocked_until = Some(now + BLOCK_DURATION);
        return Err("Слишком много попыток. Подождите 5 минут".to_string());
    }

    // Добавляем текущую попытку
    info.attempts.push(now);
    Ok(())
}

// Вызывается при успешном входе (сбрасывает счётчик)
pub fn reset_rate_limit(ip: IpAddr) {
    let mut limiter = RATE_LIMITER.lock();
    limiter.remove(&ip);
}