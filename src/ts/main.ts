interface Project {
    id: string;
    title: string;
    status: 'progress' | 'done' | 'frozen'; // Строго ограничиваем возможные статусы
    statusText: string;
    description: string;
    img: string;
    date: string;
    date_of_changes: string;
    link: string;
    resource: string;
}

interface News {
    id: string;
    title: string;
    date: string;
    description: string;
    link: string;
}

interface PersonalNews {
  id: string;
  title: string;
  date: string;
  description: string;
}


// Импорты
import projectsData from '../data/projects.json';
import newsData from '../data/news.json';
import personalNewsData from '../data/personal-news.json';

document.addEventListener('DOMContentLoaded', () => {
console.log("🚀 KYPbEP.ART: Система инициализирована");

const projectsContainer = document.getElementById('projects-container');

if (projectsContainer) {
    const sortedProjects = (projectsData as Project[])
        .sort((a, b) => new Date(b.date_of_changes).getTime() - new Date(a.date_of_changes).getTime())
        .slice(0, 3); // Показываем только 3 самых свежих

    const projectsHTML = sortedProjects.map(project => {
    // Проверяем, есть ли путь к картинке. Если да - делаем тег img, если нет - заглушку
    const imageHtml = project.img 
        ? `<img src="${project.img}" alt="${project.title}">` 
        : `<div class="placeholder-img">img: ${project.id}</div>`;

    return `
        <div class="card">
        <div class="retro-img">
            ${imageHtml}
        </div>
        <div class="card-body">
            <div class="card-top">
            <div class="card-title">${project.title}</div>
            <span class="status ${project.status}">${project.statusText}</span>
            </div>
            <p class="card-desc">${project.description}</p>
            
            <div style="font-size: 11px; color: var(--text-dim); margin-bottom: 8px; font-family: var(--font);">
            Обновлено: ${project.date_of_changes}
            </div>

            <div class="card-actions">
            <a href="${project.link}" class="btn">подробнее</a>
            <a href="#" class="resource-link">ресурс ↗</a>
            </div>
        </div>
        </div>
    `;
    }).join('');

    projectsContainer.innerHTML = projectsHTML;
}
});

const newsContainer = document.getElementById('news-container');

if (newsContainer) {
    // Сортируем новости: самые свежие (date) идут первыми
    const sortedNews = (newsData as News[])
        .sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime())
        .slice(0, 3); // Показываем только 3 самых свежих
        // Генерируем HTML для каждой новости
    const newsHTML = sortedNews.map(news => `
        <div class="news-card">
        <div class="retro-img">
            <div class="placeholder-img">img: ${news.id}</div>
        </div>
        <div class="news-body">
            <div class="news-top">
            <div class="news-title">${news.title}</div>
            <span class="news-date">${news.date}</span>
            </div>
            <p class="news-desc">${news.description}</p>
            <div class="card-actions">
            <a href="${news.link}" class="btn">подробнее</a>
            </div>
        </div>
        </div>
    `).join('');

    // Вставляем в контейнер
    newsContainer.innerHTML = newsHTML;
}

const personalNewsContainer = document.getElementById('personal-news-container');
if (personalNewsContainer) {
  const sortedPersonalNews = (personalNewsData as PersonalNews[])
    .sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime());

  const personalNewsHTML = sortedPersonalNews.map(news => `
    <div class="personal-news-item">
      <div class="personal-news-top">
        <div class="personal-news-title">${news.title}</div>
        <span class="personal-news-date">${news.date}</span>
      </div>
      <p class="personal-news-desc">${news.description}</p>
    </div>
  `).join('');

  personalNewsContainer.innerHTML = personalNewsHTML;
}

// ==========================================
// АВТОРИЗАЦИЯ И УПРАВЛЕНИЕ ПОЛЬЗОВАТЕЛЕМ
// ==========================================

const API_URL = "http://127.0.0.1:8081"; // Наш Rust-бэкенд

// Проверка авторизации при загрузке страницы
document.addEventListener('DOMContentLoaded', () => {
    checkAuthStatus();
    setupAuthForms();
});

function checkAuthStatus() {
    const token = localStorage.getItem('auth_token');
    const userStr = localStorage.getItem('auth_user');
    const container = document.getElementById('auth-container');

    if (token && userStr && container) {
        const user = JSON.parse(userStr);
        container.innerHTML = `
            <div class="user-profile" onclick="logout()">
                <img src="${user.avatar || '/src/assets/png/KYPbEP.png'}" class="user-avatar-small" alt="avatar">
                <span class="user-name">${user.login}</span>
            </div>
        `;
    } else if (container) {
        container.innerHTML = `
            <button class="btn-auth secondary" onclick="openAuthModal('login')">вход</button>
            <button class="btn-auth primary" onclick="openAuthModal('register')">регистрация</button>
        `;
    }
}

// Управление модальным окном
(window as any).openAuthModal = (tab: 'login' | 'register') => {
    const modal = document.getElementById('auth-modal');
    if (modal) modal.style.display = 'flex';
    (window as any).switchTab(tab);
};

(window as any).closeAuthModal = () => {
    const modal = document.getElementById('auth-modal');
    if (modal) modal.style.display = 'none';
    // Очистка сообщений об ошибках
    document.querySelectorAll('.form-message').forEach(el => el.textContent = '');
};

(window as any).switchTab = (tab: 'login' | 'register') => {
    const loginForm = document.getElementById('login-form');
    const registerForm = document.getElementById('register-form');
    const tabs = document.querySelectorAll('.tab-btn');

    if (tab === 'login') {
        if (loginForm) loginForm.style.display = 'flex';
        if (registerForm) registerForm.style.display = 'none';
        tabs[0].classList.add('active');
        tabs[1].classList.remove('active');
    } else {
        if (loginForm) loginForm.style.display = 'none';
        if (registerForm) registerForm.style.display = 'flex';
        tabs[0].classList.remove('active');
        tabs[1].classList.add('active');
    }
};

(window as any).logout = () => {
    localStorage.removeItem('auth_token');
    localStorage.removeItem('auth_user');
    checkAuthStatus();
};

// Обработка отправки форм
function setupAuthForms() {
    // Вход
    const loginForm = document.getElementById('login-form') as HTMLFormElement;
    if (loginForm) {
        loginForm.addEventListener('submit', async (e) => {
            e.preventDefault();
            const msgEl = document.getElementById('login-message')!;
            msgEl.textContent = 'Проверка данных...';
            msgEl.className = 'form-message';

            const payload = {
                login: (document.getElementById('login-username') as HTMLInputElement).value,
                password: (document.getElementById('login-password') as HTMLInputElement).value,
            };

            try {
                const res = await fetch(`${API_URL}/api/login`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(payload),
                });
                const data = await res.json();

                if (res.ok) {
                    localStorage.setItem('auth_token', data.token);
                    localStorage.setItem('auth_user', JSON.stringify(data.user));
                    (window as any).closeAuthModal();
                    checkAuthStatus();
                } else {
                    msgEl.textContent = data.error || 'Ошибка входа';
                    msgEl.className = 'form-message error';
                }
            } catch (err) {
                msgEl.textContent = 'Ошибка соединения с сервером';
                msgEl.className = 'form-message error';
            }
        });
    }

    // Регистрация
    const registerForm = document.getElementById('register-form') as HTMLFormElement;
    if (registerForm) {
        registerForm.addEventListener('submit', async (e) => {
            e.preventDefault();
            const msgEl = document.getElementById('reg-message')!;
            msgEl.textContent = 'Создание аккаунта...';
            msgEl.className = 'form-message';

            const payload = {
                login: (document.getElementById('reg-username') as HTMLInputElement).value,
                email: (document.getElementById('reg-email') as HTMLInputElement).value,
                password: (document.getElementById('reg-password') as HTMLInputElement).value,
            };

            try {
                const res = await fetch(`${API_URL}/api/register`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify(payload),
                });
                const data = await res.json();

                if (res.ok) {
                    localStorage.setItem('auth_token', data.token);
                    localStorage.setItem('auth_user', JSON.stringify(data.user));
                    (window as any).closeAuthModal();
                    checkAuthStatus();
                } else {
                    msgEl.textContent = data.error || 'Ошибка регистрации';
                    msgEl.className = 'form-message error';
                }
            } catch (err) {
                msgEl.textContent = 'Ошибка соединения с сервером';
                msgEl.className = 'form-message error';
            }
        });
    }
}