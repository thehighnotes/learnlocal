// LearnLocal Studio — Application Controller
'use strict';

// ── State ──────────────────────────────────────────────
const S = {
    course: null,
    lessons: [],
    currentView: 'welcome',    // welcome | course | lesson | exercise | preview
    currentLesson: null,       // lesson id
    currentExercise: null,     // exercise id
    exerciseData: null,        // loaded exercise object
    activeExTab: 'prompt',     // prompt | solution | starter | validate | hints | explanation | stages
    activeLessonTab: 'content',// content | exercises | settings
    saveTimer: null,
    terminal: null,
    previewWs: null,
};

// ── API ────────────────────────────────────────────────
async function api(method, path, body) {
    const opts = { method, headers: { 'Content-Type': 'application/json' } };
    if (body) opts.body = JSON.stringify(body);
    const r = await fetch(`/api${path}`, opts);
    if (!r.ok) { const e = await r.json().catch(() => ({})); throw new Error(e.error || r.statusText); }
    if (r.status === 204 || r.headers.get('content-length') === '0') return null;
    return r.json();
}

function toast(msg, type = 'info') {
    const c = document.getElementById('toast-container');
    const el = document.createElement('div');
    el.className = `toast ${type}`;
    el.textContent = msg;
    c.appendChild(el);
    setTimeout(() => el.remove(), 3000);
}

function setStatus(text, type = '') {
    const el = document.getElementById('status');
    el.textContent = text;
    el.className = `status-pill ${type}`;
}

function esc(s) { return String(s || '').replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/"/g,'&quot;'); }

// ── Modal System ───────────────────────────────────────
function showModal(opts) {
    // opts: { title, body (html string), onSubmit (fn receiving formData), submitLabel, cancelLabel }
    return new Promise(resolve => {
        const overlay = document.createElement('div');
        overlay.className = 'modal-overlay';
        overlay.onclick = e => { if (e.target === overlay) { overlay.remove(); resolve(null); } };

        const modal = document.createElement('div');
        modal.className = 'modal';
        modal.innerHTML = `
            <div class="modal-header">
                <span>${opts.title || ''}</span>
                <button class="btn btn-sm btn-ghost modal-close">✕</button>
            </div>
            <form class="modal-body">${opts.body || ''}</form>
            <div class="modal-footer">
                <button type="button" class="btn btn-sm modal-cancel">${opts.cancelLabel || 'Cancel'}</button>
                <button type="button" class="btn btn-sm btn-primary modal-submit">${opts.submitLabel || 'Create'}</button>
            </div>
        `;

        modal.querySelector('.modal-close').onclick = () => { overlay.remove(); resolve(null); };
        modal.querySelector('.modal-cancel').onclick = () => { overlay.remove(); resolve(null); };
        modal.querySelector('.modal-submit').onclick = () => {
            const form = modal.querySelector('form');
            const data = Object.fromEntries(new FormData(form));
            overlay.remove();
            resolve(data);
        };
        // Enter key submits
        modal.querySelector('form').onkeydown = e => {
            if (e.key === 'Enter' && e.target.tagName !== 'TEXTAREA') {
                e.preventDefault();
                modal.querySelector('.modal-submit').click();
            }
        };

        overlay.appendChild(modal);
        document.body.appendChild(overlay);

        // Focus first input
        const first = modal.querySelector('input, select');
        if (first) setTimeout(() => first.focus(), 50);
    });
}

function showConfirm(opts) {
    // opts: { title, message, confirmLabel, danger }
    return new Promise(resolve => {
        const overlay = document.createElement('div');
        overlay.className = 'modal-overlay';
        overlay.onclick = e => { if (e.target === overlay) { overlay.remove(); resolve(false); } };

        const modal = document.createElement('div');
        modal.className = 'modal modal-sm';
        modal.innerHTML = `
            <div class="modal-header">
                <span>${opts.title || 'Confirm'}</span>
                <button class="btn btn-sm btn-ghost modal-close">✕</button>
            </div>
            <div class="modal-body"><p style="margin:0;line-height:1.6">${opts.message || 'Are you sure?'}</p></div>
            <div class="modal-footer">
                <button class="btn btn-sm modal-cancel">Cancel</button>
                <button class="btn btn-sm ${opts.danger ? 'btn-danger' : 'btn-primary'} modal-confirm">${opts.confirmLabel || 'Confirm'}</button>
            </div>
        `;

        modal.querySelector('.modal-close').onclick = () => { overlay.remove(); resolve(false); };
        modal.querySelector('.modal-cancel').onclick = () => { overlay.remove(); resolve(false); };
        modal.querySelector('.modal-confirm').onclick = () => { overlay.remove(); resolve(true); };

        overlay.appendChild(modal);
        document.body.appendChild(overlay);
        modal.querySelector('.modal-confirm').focus();
    });
}

// ── Navigation ─────────────────────────────────────────
function navigate(view, opts = {}) {
    S.currentView = view;
    if (opts.lesson !== undefined) S.currentLesson = opts.lesson;
    if (opts.exercise !== undefined) S.currentExercise = opts.exercise;

    // Update rail
    document.querySelectorAll('.rail-btn').forEach(b => b.classList.remove('active'));
    const rb = document.querySelector(`.rail-btn[data-view="${view}"]`);
    if (rb) rb.classList.add('active');

    // Enable/disable rail buttons
    document.getElementById('rail-lesson').disabled = !S.currentLesson;
    document.getElementById('rail-exercise').disabled = !S.currentExercise;

    // Show view
    document.querySelectorAll('.view').forEach(v => v.classList.remove('active'));
    const target = document.getElementById(`view-${view}`);
    if (target) target.classList.add('active');

    // Render
    updateBreadcrumb();
    if (document.getElementById('help-panel').classList.contains('open')) updateHelpContent();
    if (view === 'welcome') renderWelcome();
    else if (view === 'course') renderCourse();
    else if (view === 'lesson') renderLesson();
    else if (view === 'exercise') loadAndRenderExercise();
    else if (view === 'preview') initPreview();
}

function updateBreadcrumb() {
    const bc = document.getElementById('breadcrumb');
    let parts = [];

    parts.push({ label: S.course?.name || 'Course', action: () => navigate('course') });

    if (S.currentLesson) {
        const lesson = S.lessons.find(l => l.id === S.currentLesson);
        parts.push({ label: lesson?.title || S.currentLesson, action: () => navigate('lesson', { lesson: S.currentLesson }) });
    }
    if (S.currentExercise && S.currentView === 'exercise') {
        parts.push({ label: S.currentExercise, action: null });
    }

    bc.innerHTML = parts.map((p, i) => {
        const isLast = i === parts.length - 1;
        const chevron = i > 0 ? '<span class="chevron">›</span>' : '';
        if (isLast) return `${chevron}<span class="crumb current">${esc(p.label)}</span>`;
        return `${chevron}<span class="crumb" onclick="navigate('${i === 0 ? 'course' : 'lesson'}', ${i === 0 ? '{}' : `{lesson:'${S.currentLesson}'}`})">${esc(p.label)}</span>`;
    }).join('');
}

// ── Welcome View ───────────────────────────────────────
async function renderWelcome() {
    const el = document.getElementById('view-welcome');
    let projectsHtml = '<p style="color:var(--text-3)">Loading projects...</p>';

    el.innerHTML = `
        <div style="max-width:720px;margin:0 auto;padding-top:40px">
            <div style="text-align:center;margin-bottom:48px">
                <h1 style="font-size:28px;font-weight:700;letter-spacing:-0.03em;margin-bottom:8px">
                    <span style="background:linear-gradient(135deg,#8b5cf6,#c084fc);-webkit-background-clip:text;-webkit-text-fill-color:transparent">LearnLocal Studio</span>
                </h1>
                <p style="color:var(--text-2);font-size:15px">Create, edit, and publish programming courses</p>
            </div>

            <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;margin-bottom:40px">
                <button class="btn" style="padding:20px;flex-direction:column;gap:8px;justify-content:center;height:auto" onclick="showNewCourseDialog()">
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="16"/><line x1="8" y1="12" x2="16" y2="12"/></svg>
                    <span style="font-size:14px;font-weight:600;color:var(--text-0)">New Course</span>
                    <span style="font-size:12px;color:var(--text-2)">Start from scratch</span>
                </button>
                <button class="btn" style="padding:20px;flex-direction:column;gap:8px;justify-content:center;height:auto" onclick="showOpenDialog()">
                    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/></svg>
                    <span style="font-size:14px;font-weight:600;color:var(--text-0)">Open Course</span>
                    <span style="font-size:12px;color:var(--text-2)">Load existing project</span>
                </button>
            </div>

            <div class="section-label"><span>Recent Projects</span></div>
            <div id="project-list">${projectsHtml}</div>
        </div>
    `;

    // Load projects async
    try {
        const projects = await api('GET', '/workspace/projects');
        const list = document.getElementById('project-list');
        if (!projects.length) {
            list.innerHTML = `
                <div class="empty" style="padding:30px">
                    <h3>No projects yet</h3>
                    <p style="color:var(--text-3)">Create a new course or open an existing one to get started</p>
                </div>
            `;
        } else {
            list.innerHTML = `
                <table class="data-table">
                    <thead><tr>
                        <th>Name</th>
                        <th style="width:90px">Status</th>
                        <th style="width:60px">Lessons</th>
                        <th style="width:100px">Authors</th>
                        <th style="width:30px"></th>
                    </tr></thead>
                    <tbody>
                        ${projects.map(p => `
                            <tr onclick="openProject('${esc(p.path)}')">
                                <td class="title-cell">
                                    ${esc(p.name)}
                                    <span style="color:var(--text-3);font-size:12px;margin-left:6px">v${esc(p.version)}</span>
                                </td>
                                <td><span class="type-badge ${p.status}">${esc(p.status)}</span></td>
                                <td>${p.lesson_count}</td>
                                <td style="color:var(--text-2);font-size:12px">${p.authors.length ? esc(p.authors.join(', ')) : '—'}</td>
                                <td><span class="row-action">→</span></td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            `;
        }
    } catch (e) {
        document.getElementById('project-list').innerHTML = `<p style="color:var(--error)">${esc(e.message)}</p>`;
    }
}

async function openProject(path) {
    try {
        await api('POST', '/workspace/open', { path });
        S.course = await api('GET', '/course');
        document.title = `${S.course.name} — LearnLocal Studio`;
        navigate('course');
        toast('Project opened', 'success');
    } catch (e) { toast(e.message, 'error'); }
}

const LANGUAGES = [
    { id: 'cpp', display: 'C++', ext: '.cpp' },
    { id: 'python', display: 'Python', ext: '.py' },
    { id: 'javascript', display: 'JavaScript', ext: '.js' },
    { id: 'rust', display: 'Rust', ext: '.rs' },
    { id: 'go', display: 'Go', ext: '.go' },
    { id: 'java', display: 'Java', ext: '.java' },
    { id: 'sql', display: 'SQL', ext: '.sql' },
    { id: 'bash', display: 'Bash', ext: '.sh' },
];

async function showNewCourseDialog() {
    const data = await showModal({
        title: 'New Course',
        submitLabel: 'Create Course',
        body: `
            <div class="field">
                <label>Course Name</label>
                <input name="name" placeholder="e.g. Python Fundamentals" required>
            </div>
            <div class="field">
                <label>Language</label>
                <select name="language">
                    ${LANGUAGES.map(l => `<option value="${l.id}">${l.display}</option>`).join('')}
                </select>
            </div>
        `,
    });
    if (!data || !data.name) return;

    const lang = LANGUAGES.find(l => l.id === data.language);
    if (!lang) return;
    createNewCourse(data.name, lang);
}

async function createNewCourse(name, lang) {
    try {
        await api('POST', '/workspace/create', {
            name,
            language_id: lang.id,
            language_display: lang.display,
            extension: lang.ext,
        });
        S.course = await api('GET', '/course');
        document.title = `${S.course.name} — LearnLocal Studio`;
        navigate('course');
        toast('Course created!', 'success');
    } catch (e) { toast(e.message, 'error'); }
}

async function showOpenDialog() {
    try {
        const projects = await api('GET', '/workspace/projects');
        if (!projects.length) {
            toast('No courses found. Create a new one instead.', 'info');
            return;
        }

        // Build a modal picker
        const overlay = document.createElement('div');
        overlay.style.cssText = 'position:fixed;inset:0;background:rgba(0,0,0,0.6);z-index:500;display:flex;align-items:center;justify-content:center;backdrop-filter:blur(4px)';
        overlay.onclick = e => { if (e.target === overlay) overlay.remove(); };

        const modal = document.createElement('div');
        modal.style.cssText = 'background:var(--bg-2);border:1px solid var(--border-1);border-radius:var(--r-lg);width:560px;max-height:70vh;overflow:hidden;display:flex;flex-direction:column;box-shadow:0 16px 64px rgba(0,0,0,0.5)';
        modal.innerHTML = `
            <div style="padding:18px 22px;border-bottom:1px solid var(--border-0);font-weight:600;font-size:15px;display:flex;align-items:center;justify-content:space-between">
                <span>Open Course</span>
                <button class="btn btn-sm btn-ghost" onclick="this.closest('[style*=fixed]').remove()">✕</button>
            </div>
            <div style="overflow-y:auto;flex:1">
                ${projects.map(p => `
                    <div style="padding:14px 22px;border-bottom:1px solid var(--border-0);cursor:pointer;transition:background 150ms ease"
                         onmouseenter="this.style.background='var(--bg-3)'" onmouseleave="this.style.background=''"
                         onclick="this.closest('[style*=fixed]').remove(); openProject('${esc(p.path)}')">
                        <div style="font-weight:500;margin-bottom:2px">${esc(p.name)} <span style="color:var(--text-3);font-size:12px">v${esc(p.version)}</span></div>
                        <div style="font-size:12px;color:var(--text-2)">${p.lesson_count} lessons · ${p.exercise_count} exercises${p.authors.length ? ' · ' + esc(p.authors.join(', ')) : ''}</div>
                        <div style="font-size:11px;color:var(--text-3);margin-top:2px;font-family:var(--mono)">${esc(p.path)}</div>
                    </div>
                `).join('')}
            </div>
        `;

        overlay.appendChild(modal);
        document.body.appendChild(overlay);
    } catch (e) {
        toast('Failed to list projects: ' + e.message, 'error');
    }
}

// ── Course View ────────────────────────────────────────
async function renderCourse() {
    S.lessons = await api('GET', '/lessons');
    const c = S.course;
    const el = document.getElementById('view-course');

    el.innerHTML = `
        <div class="view-header">
            <div>
                <h2>${esc(c.name)}</h2>
                <span class="subtitle">v${esc(c.version)} · ${S.lessons.length} lessons · ${S.lessons.reduce((a, l) => a + l.exercise_count, 0)} exercises</span>
            </div>
            <div class="view-actions">
                <button class="btn btn-sm" onclick="validateCourse()">Validate All</button>
                <button class="btn btn-sm btn-primary" onclick="addLesson()">+ Lesson</button>
            </div>
        </div>

        <div class="meta-grid">
            <div class="meta-card">
                <label>Name</label>
                <input value="${esc(c.name)}" onchange="updateCourseMeta('name', this.value)">
            </div>
            <div class="meta-card">
                <label>Version</label>
                <input value="${esc(c.version)}" onchange="updateCourseMeta('version', this.value)">
            </div>
            <div class="meta-card">
                <label>Author</label>
                <input value="${esc(c.author)}" onchange="updateCourseMeta('author', this.value)">
            </div>
            <div class="meta-card">
                <label>License</label>
                <input value="${esc(c.license || '')}" onchange="updateCourseMeta('license', this.value)">
            </div>
            <div class="meta-card full">
                <label>Description</label>
                <input value="${esc(c.description)}" onchange="updateCourseMeta('description', this.value)">
            </div>
        </div>

        <div class="section-label">
            <span>Lessons</span>
        </div>

        <table class="data-table" id="lesson-table">
            <thead><tr>
                <th style="width:30px"></th>
                <th>Title</th>
                <th style="width:80px">Exercises</th>
                <th style="width:100px">Status</th>
                <th style="width:30px"></th>
            </tr></thead>
            <tbody>
                ${S.lessons.map(l => `
                    <tr data-id="${esc(l.id)}" onclick="navigate('lesson', {lesson:'${esc(l.id)}'})">
                        <td><span class="grip">⠿</span></td>
                        <td class="title-cell">${esc(l.title)}</td>
                        <td>${l.exercise_count}</td>
                        <td><span class="status-tag pass">✓ Valid</span></td>
                        <td><span class="row-action">→</span></td>
                    </tr>
                `).join('')}
            </tbody>
        </table>
    `;

    // Make lessons sortable
    const tbody = el.querySelector('#lesson-table tbody');
    if (tbody) {
        new Sortable(tbody, {
            handle: '.grip', animation: 200,
            onEnd: async () => {
                const order = [...tbody.children].map(r => r.dataset.id);
                try { await api('PUT', '/lessons/reorder', { order }); toast('Reordered', 'success'); }
                catch (e) { toast(e.message, 'error'); renderCourse(); }
            }
        });
    }
}

async function updateCourseMeta(field, value) {
    try {
        await api('PUT', '/course', { [field]: value });
        S.course[field] = value;
        document.querySelector('.course-name')?.textContent;
        setStatus('Saved', 'saved');
        setTimeout(() => setStatus('Ready'), 1500);
    } catch (e) { toast(e.message, 'error'); }
}

async function addLesson() {
    const data = await showModal({
        title: 'New Lesson',
        submitLabel: 'Create Lesson',
        body: `
            <div class="field">
                <label>Lesson Title</label>
                <input name="title" placeholder="e.g. Variables and Types" required>
            </div>
            <div class="field">
                <label>Lesson ID</label>
                <input name="id" placeholder="e.g. variables" required pattern="[a-z0-9\\-]+">
                <div style="font-size:11px;color:var(--text-3);margin-top:4px">Lowercase, hyphens only. Used as the directory name.</div>
            </div>
        `,
    });
    if (!data || !data.id || !data.title) return;
    try {
        await api('POST', '/lessons', { id: data.id, title: data.title });
        toast('Lesson created', 'success');
        renderCourse();
    } catch (e) { toast(e.message, 'error'); }
}

async function deleteLesson(id) {
    const ok = await showConfirm({
        title: 'Delete Lesson',
        message: `Delete lesson <strong>"${esc(id)}"</strong> and all its exercises? This cannot be undone.`,
        confirmLabel: 'Delete',
        danger: true,
    });
    if (!ok) return;
    try {
        await api('DELETE', `/lessons/${id}`);
        toast('Deleted', 'success');
        renderCourse();
    } catch (e) { toast(e.message, 'error'); }
}

// ── Lesson View ────────────────────────────────────────
async function renderLesson() {
    const lesson = S.lessons.find(l => l.id === S.currentLesson);
    if (!lesson) return navigate('course');

    const el = document.getElementById('view-lesson');

    el.innerHTML = `
        <div class="view-header">
            <div>
                <h2>${esc(lesson.title)}</h2>
                <span class="subtitle">${lesson.exercise_count} exercises</span>
            </div>
            <div class="view-actions">
                <button class="btn btn-sm btn-danger" onclick="deleteLesson('${esc(lesson.id)}')">Delete Lesson</button>
                <button class="btn btn-sm btn-primary" onclick="addExercise('${esc(lesson.id)}')">+ Exercise</button>
            </div>
        </div>

        <div class="tabs" id="lesson-tabs">
            <button class="tab ${S.activeLessonTab === 'exercises' ? 'active' : ''}" onclick="switchLessonTab('exercises')">
                Exercises<span class="tab-badge">${lesson.exercise_count}</span>
            </button>
            <button class="tab ${S.activeLessonTab === 'settings' ? 'active' : ''}" onclick="switchLessonTab('settings')">Settings</button>
        </div>

        <div id="lesson-tab-exercises" class="tab-content ${S.activeLessonTab === 'exercises' ? 'active' : ''}">
            ${renderExerciseTable(lesson)}
        </div>
        <div id="lesson-tab-settings" class="tab-content ${S.activeLessonTab === 'settings' ? 'active' : ''}">
            <div class="field">
                <label>Lesson ID</label>
                <input value="${esc(lesson.id)}" readonly style="opacity:0.5">
            </div>
            <div class="field">
                <label>Title</label>
                <input value="${esc(lesson.title)}">
            </div>
        </div>
    `;

    // Make exercises sortable
    const tbody = el.querySelector('#exercise-table tbody');
    if (tbody) {
        new Sortable(tbody, {
            handle: '.grip', animation: 200,
            onEnd: async () => {
                const order = [...tbody.children].map(r => r.dataset.id);
                try {
                    await api('PUT', `/lessons/${lesson.id}/exercises/reorder`, { order });
                    toast('Reordered', 'success');
                } catch (e) { toast(e.message, 'error'); renderLesson(); }
            }
        });
    }
}

function renderExerciseTable(lesson) {
    if (!lesson.exercises.length) {
        return `<div class="empty"><h3>No exercises yet</h3><p>Add your first exercise to get started</p></div>`;
    }
    return `
        <table class="data-table" id="exercise-table">
            <thead><tr>
                <th style="width:30px"></th>
                <th>Title</th>
                <th style="width:90px">Type</th>
                <th style="width:30px"></th>
            </tr></thead>
            <tbody>
                ${lesson.exercises.map(ex => `
                    <tr data-id="${esc(ex.id)}" onclick="navigate('exercise', {lesson:'${esc(lesson.id)}',exercise:'${esc(ex.id)}'})">
                        <td><span class="grip">⠿</span></td>
                        <td class="title-cell">
                            ${esc(ex.title)}
                            ${ex.has_stages ? '<span class="type-badge staged">staged</span>' : ''}
                        </td>
                        <td><span class="type-badge ${ex.type}">${esc(ex.type)}</span></td>
                        <td><span class="row-action">→</span></td>
                    </tr>
                `).join('')}
            </tbody>
        </table>
    `;
}

function switchLessonTab(tab) {
    S.activeLessonTab = tab;
    document.querySelectorAll('#lesson-tabs .tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('#view-lesson .tab-content').forEach(t => t.classList.remove('active'));
    document.querySelector(`#lesson-tabs .tab:nth-child(${tab === 'exercises' ? 1 : 2})`).classList.add('active');
    document.getElementById(`lesson-tab-${tab}`).classList.add('active');
}

async function addExercise(lessonId) {
    const types = ['write','fix','fill-blank','command','predict','multiple-choice'];
    const data = await showModal({
        title: 'New Exercise',
        submitLabel: 'Create Exercise',
        body: `
            <div class="field">
                <label>Exercise Title</label>
                <input name="title" placeholder="e.g. Hello, World!" required>
            </div>
            <div class="field-row">
                <div class="field">
                    <label>Exercise ID</label>
                    <input name="id" placeholder="e.g. hello-world" required pattern="[a-z0-9\\-]+">
                </div>
                <div class="field">
                    <label>Type</label>
                    <select name="type">
                        ${types.map(t => `<option value="${t}">${t}</option>`).join('')}
                    </select>
                </div>
            </div>
        `,
    });
    if (!data || !data.id || !data.title) return;
    try {
        await api('POST', `/lessons/${lessonId}/exercises`, {
            id: data.id, title: data.title, type: data.type || 'write',
            prompt: '', starter: '// Your code here\n',
            validation: { method: 'output', expected_output: '' }, hints: [], solution: '',
        });
        S.lessons = await api('GET', '/lessons');
        toast('Exercise created', 'success');
        navigate('exercise', { lesson: lessonId, exercise: data.id });
    } catch (e) { toast(e.message, 'error'); }
}

// ── Exercise View ──────────────────────────────────────
async function loadAndRenderExercise() {
    if (!S.currentLesson || !S.currentExercise) return navigate('course');
    try {
        S.exerciseData = await api('GET', `/lessons/${S.currentLesson}/exercises/${S.currentExercise}`);
        renderExercise();
    } catch (e) { toast(e.message, 'error'); navigate('lesson', { lesson: S.currentLesson }); }
}

function renderExercise() {
    const ex = S.exerciseData;
    if (!ex) return;
    const el = document.getElementById('view-exercise');
    const types = ['write','fix','fill-blank','multiple-choice','predict','command'];
    const hasStages = ex.stages && ex.stages.length > 0;

    const tabs = [
        { id: 'prompt', label: 'Prompt' },
        { id: 'solution', label: 'Solution' },
        { id: 'starter', label: 'Starter' },
        { id: 'validate', label: 'Validate', badge: null },
        { id: 'hints', label: 'Hints', badge: (ex.hints || []).length },
        { id: 'explanation', label: 'Explanation' },
    ];
    if (hasStages) tabs.push({ id: 'stages', label: 'Stages', badge: ex.stages.length });

    el.innerHTML = `
        <div class="view-header">
            <div>
                <h2>${esc(ex.title)}</h2>
                <span class="subtitle">${esc(ex.id)}</span>
            </div>
            <div class="view-actions">
                <div class="field-row" style="margin:0;gap:8px;align-items:center">
                    <select id="ex-type" style="width:130px;padding:6px 10px;font-size:12px" onchange="scheduleAutoSave()">
                        ${types.map(t => `<option value="${t}" ${ex.type === t ? 'selected' : ''}>${t}</option>`).join('')}
                    </select>
                </div>
                <button class="btn btn-sm btn-primary" onclick="saveExercise()">Save</button>
                <button class="btn btn-sm btn-danger" onclick="deleteExercise()">Delete</button>
            </div>
        </div>

        <div class="tabs" id="ex-tabs">
            ${tabs.map(t => `
                <button class="tab ${S.activeExTab === t.id ? 'active' : ''}" onclick="switchExTab('${t.id}')">
                    ${t.label}${t.badge != null ? `<span class="tab-badge">${t.badge}</span>` : ''}
                </button>
            `).join('')}
        </div>

        <div id="ex-tab-prompt" class="tab-content ${S.activeExTab === 'prompt' ? 'active' : ''}">
            <div class="field">
                <label>Title</label>
                <input id="ex-title" value="${esc(ex.title)}" onchange="scheduleAutoSave()">
            </div>
            <div class="field">
                <label>Prompt (what the student sees)</label>
                <textarea id="ex-prompt" rows="8" onchange="scheduleAutoSave()">${esc(ex.prompt)}</textarea>
            </div>
        </div>

        <div id="ex-tab-solution" class="tab-content ${S.activeExTab === 'solution' ? 'active' : ''}">
            <div class="code-block">
                <div class="code-block-header">
                    <span>Solution code</span>
                    <button class="btn btn-sm" onclick="runSolution()">▶ Run Solution</button>
                </div>
                <textarea id="ex-solution" onchange="scheduleAutoSave()">${esc(ex.solution || '')}</textarea>
            </div>
            <div id="solution-output" style="display:none"></div>
        </div>

        <div id="ex-tab-starter" class="tab-content ${S.activeExTab === 'starter' ? 'active' : ''}">
            <div class="code-block">
                <div class="code-block-header">
                    <span>Starter code (what the student begins with)</span>
                </div>
                <textarea id="ex-starter" onchange="scheduleAutoSave()">${esc(ex.starter || '')}</textarea>
            </div>
        </div>

        <div id="ex-tab-validate" class="tab-content ${S.activeExTab === 'validate' ? 'active' : ''}">
            ${renderValidateTab(ex)}
        </div>

        <div id="ex-tab-hints" class="tab-content ${S.activeExTab === 'hints' ? 'active' : ''}">
            ${renderHintsTab(ex)}
        </div>

        <div id="ex-tab-explanation" class="tab-content ${S.activeExTab === 'explanation' ? 'active' : ''}">
            <div class="field">
                <label>Explanation (shown after student passes)</label>
                <textarea id="ex-explanation" rows="6" onchange="scheduleAutoSave()">${esc(ex.explanation || '')}</textarea>
            </div>
        </div>

        ${hasStages ? `<div id="ex-tab-stages" class="tab-content ${S.activeExTab === 'stages' ? 'active' : ''}">${renderStagesTab(ex)}</div>` : ''}
    `;

    // Wire auto-save
    el.querySelectorAll('input:not([readonly]),textarea,select').forEach(i => {
        i.addEventListener('input', scheduleAutoSave);
    });
}

function renderValidateTab(ex) {
    const methods = ['output','regex','compile-only','state','custom'];
    const method = ex.validation?.method || 'output';
    return `
        <div class="field">
            <label>Validation Method</label>
            <div class="method-pills">
                ${methods.map(m => `
                    <button class="method-pill ${method === m ? 'active' : ''}"
                            onclick="setValidationMethod('${m}')">${m}</button>
                `).join('')}
            </div>
        </div>
        <div id="val-fields">
            ${method === 'output' ? `
                <div class="field">
                    <label>Expected Output</label>
                    <input id="ex-val-expected" value="${esc(ex.validation?.expected_output || '')}" onchange="scheduleAutoSave()">
                </div>
            ` : ''}
            ${method === 'regex' ? `
                <div class="field">
                    <label>Pattern</label>
                    <input id="ex-val-pattern" value="${esc(ex.validation?.pattern || '')}" onchange="scheduleAutoSave()"
                           style="font-family:var(--mono)">
                </div>
            ` : ''}
        </div>
        <div style="margin-top:16px">
            <button class="btn btn-sm" onclick="testValidation()">▶ Test Against Solution</button>
        </div>
        <div id="val-result"></div>
    `;
}

function setValidationMethod(method) {
    if (!S.exerciseData) return;
    S.exerciseData.validation = S.exerciseData.validation || {};
    S.exerciseData.validation.method = method;
    renderExercise();
    scheduleAutoSave();
}

function renderHintsTab(ex) {
    const hints = ex.hints || [];
    return `
        <ul class="hint-list" id="hint-list">
            ${hints.map((h, i) => `
                <li class="hint-item" data-idx="${i}">
                    <span class="grip">⠿</span>
                    <input value="${esc(h)}" onchange="scheduleAutoSave()">
                    <button class="remove-btn" onclick="removeHint(${i})">×</button>
                </li>
            `).join('')}
        </ul>
        ${!hints.length ? '<p style="color:var(--text-3);margin-bottom:12px">No hints yet. Students rely on hints when stuck.</p>' : ''}
        <button class="btn btn-sm" onclick="addHint()">+ Add Hint</button>
    `;
}

function renderStagesTab(ex) {
    return `
        ${(ex.stages || []).map((s, i) => `
            <div class="stage-card">
                <div class="stage-card-hdr" onclick="this.parentElement.classList.toggle('collapsed')">
                    <span>Stage ${i + 1}: ${esc(s.title || s.id)}</span>
                    <span style="color:var(--text-3);font-size:12px">${esc(s.id)}</span>
                </div>
                <div class="stage-card-body">
                    <div class="field"><label>ID</label><input value="${esc(s.id)}" data-stage="${i}" data-field="id" onchange="scheduleAutoSave()"></div>
                    <div class="field"><label>Title</label><input value="${esc(s.title)}" data-stage="${i}" data-field="title" onchange="scheduleAutoSave()"></div>
                    <div class="field"><label>Prompt</label><textarea rows="2" data-stage="${i}" data-field="prompt" onchange="scheduleAutoSave()">${esc(s.prompt || '')}</textarea></div>
                    <div class="field"><label>Solution</label><textarea rows="4" style="font-family:var(--mono)" data-stage="${i}" data-field="solution" onchange="scheduleAutoSave()">${esc(s.solution || '')}</textarea></div>
                    <div class="field"><label>Explanation</label><textarea rows="2" data-stage="${i}" data-field="explanation" onchange="scheduleAutoSave()">${esc(s.explanation || '')}</textarea></div>
                </div>
            </div>
        `).join('')}
        <button class="btn btn-sm" onclick="addStage()">+ Add Stage</button>
    `;
}

function switchExTab(tab) {
    S.activeExTab = tab;
    document.querySelectorAll('#ex-tabs .tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('#view-exercise .tab-content').forEach(t => t.classList.remove('active'));
    const idx = [...document.querySelectorAll('#ex-tabs .tab')].findIndex(t => t.textContent.trim().toLowerCase().startsWith(tab));
    if (idx >= 0) document.querySelectorAll('#ex-tabs .tab')[idx].classList.add('active');
    const tabEl = document.getElementById(`ex-tab-${tab}`);
    if (tabEl) tabEl.classList.add('active');
}

// ── Auto-save ──────────────────────────────────────────
function scheduleAutoSave() {
    if (S.saveTimer) clearTimeout(S.saveTimer);
    setStatus('Unsaved...', 'saving');
    S.saveTimer = setTimeout(saveExercise, 800);
}

async function saveExercise() {
    if (!S.currentLesson || !S.currentExercise || !S.exerciseData) return;
    const data = collectExerciseData();
    try {
        await api('PUT', `/lessons/${S.currentLesson}/exercises/${S.currentExercise}`, data);
        S.exerciseData = data;
        setStatus('Saved', 'saved');
        setTimeout(() => setStatus('Ready'), 2000);
    } catch (e) {
        setStatus('Error', 'error');
        toast('Save failed: ' + e.message, 'error');
    }
}

function collectExerciseData() {
    const ex = { ...S.exerciseData };
    ex.title = document.getElementById('ex-title')?.value || ex.title;
    ex.type = document.getElementById('ex-type')?.value || ex.type;
    ex.prompt = document.getElementById('ex-prompt')?.value ?? ex.prompt;
    ex.solution = document.getElementById('ex-solution')?.value || null;
    ex.starter = document.getElementById('ex-starter')?.value || null;
    ex.explanation = document.getElementById('ex-explanation')?.value || null;

    // Validation
    ex.validation = ex.validation || {};
    const expected = document.getElementById('ex-val-expected');
    const pattern = document.getElementById('ex-val-pattern');
    if (expected) ex.validation.expected_output = expected.value || null;
    if (pattern) ex.validation.pattern = pattern.value || null;

    // Hints
    const hintInputs = document.querySelectorAll('#hint-list .hint-item input');
    if (hintInputs.length || document.getElementById('hint-list')) {
        ex.hints = [...hintInputs].map(el => el.value).filter(v => v.trim());
    }

    // Stages
    document.querySelectorAll('.stage-card').forEach((card, i) => {
        if (!ex.stages?.[i]) return;
        card.querySelectorAll('[data-stage][data-field]').forEach(el => {
            ex.stages[i][el.dataset.field] = el.value || null;
        });
    });

    // Clean nulls
    if (!ex.solution) delete ex.solution;
    if (!ex.starter) delete ex.starter;
    if (!ex.explanation) delete ex.explanation;
    if (ex.validation && !ex.validation.expected_output) delete ex.validation.expected_output;
    if (ex.validation && !ex.validation.pattern) delete ex.validation.pattern;

    return ex;
}

// ── Actions ────────────────────────────────────────────
async function runSolution() {
    if (!S.currentLesson || !S.currentExercise) return;
    setStatus('Running...', 'saving');
    try {
        const r = await api('POST', '/run-solution', { lesson: S.currentLesson, exercise: S.currentExercise });
        const area = document.getElementById('solution-output');
        area.style.display = '';
        area.innerHTML = `
            <div class="output-panel ${r.success ? 'pass' : 'fail'}">${esc(r.stdout || r.stderr || '(no output)')}</div>
            ${r.success && r.stdout ? `<button class="btn btn-sm" style="margin-top:8px" onclick="useAsExpected()">Use as expected output</button>` : ''}
        `;
        setStatus(r.success ? 'Passes' : 'Fails', r.success ? 'saved' : 'error');
    } catch (e) { toast(e.message, 'error'); setStatus('Error', 'error'); }
}

function useAsExpected() {
    const text = document.querySelector('#solution-output .output-panel')?.textContent;
    if (!text || !S.exerciseData) return;
    S.exerciseData.validation = S.exerciseData.validation || {};
    S.exerciseData.validation.method = 'output';
    S.exerciseData.validation.expected_output = text.trim();
    S.activeExTab = 'validate';
    renderExercise();
    scheduleAutoSave();
    toast('Expected output set', 'success');
}

async function testValidation() {
    await saveExercise();
    setStatus('Testing...', 'saving');
    try {
        const r = await api('POST', `/validate/${S.currentLesson}/${S.currentExercise}`);
        const area = document.getElementById('val-result');
        area.innerHTML = `
            <div class="live-check ${r.success ? 'pass' : 'fail'}">
                <span class="dot ${r.success ? 'pass' : 'fail'}"></span>
                ${r.success ? 'Solution passes validation' : `Solution fails — stdout: "${esc(r.stdout?.trim())}"`}
            </div>
        `;
        setStatus(r.success ? 'Valid' : 'Fails', r.success ? 'saved' : 'error');
    } catch (e) { toast(e.message, 'error'); }
}

async function deleteExercise() {
    const ok = await showConfirm({
        title: 'Delete Exercise',
        message: `Delete exercise <strong>"${esc(S.currentExercise)}"</strong>? This cannot be undone.`,
        confirmLabel: 'Delete',
        danger: true,
    });
    if (!ok) return;
    try {
        await api('DELETE', `/lessons/${S.currentLesson}/exercises/${S.currentExercise}`);
        S.currentExercise = null;
        S.exerciseData = null;
        toast('Deleted', 'success');
        navigate('lesson', { lesson: S.currentLesson });
    } catch (e) { toast(e.message, 'error'); }
}

async function validateCourse() {
    setStatus('Validating...', 'saving');
    try {
        const r = await api('POST', '/validate');
        const failed = r.checks.filter(c => !c.passed);
        if (r.all_passed) {
            toast(`All ${r.checks.length} checks pass`, 'success');
            setStatus('Valid', 'saved');
        } else {
            toast(`${failed.length} issue(s) found`, 'error');
            setStatus(`${failed.length} issues`, 'error');
            console.table(failed);
        }
    } catch (e) { toast(e.message, 'error'); setStatus('Error', 'error'); }
}

function addHint() {
    if (!S.exerciseData) return;
    S.exerciseData.hints = S.exerciseData.hints || [];
    S.exerciseData.hints.push('');
    renderExercise();
}

function removeHint(i) {
    if (!S.exerciseData?.hints) return;
    S.exerciseData.hints.splice(i, 1);
    renderExercise();
    scheduleAutoSave();
}

function addStage() {
    if (!S.exerciseData) return;
    S.exerciseData.stages = S.exerciseData.stages || [];
    S.exerciseData.stages.push({
        id: `stage-${S.exerciseData.stages.length + 1}`,
        title: `Stage ${S.exerciseData.stages.length + 1}`,
        prompt: '', solution: '', explanation: '',
        validation: { method: 'output', expected_output: '' }, hints: [],
    });
    S.activeExTab = 'stages';
    renderExercise();
    scheduleAutoSave();
}

// ── Terminal Preview ───────────────────────────────────
function initPreview() {
    const container = document.getElementById('terminal-container');
    container.innerHTML = '';
    if (S.terminal) { S.terminal.dispose(); S.terminal = null; }
    if (S.previewWs) { S.previewWs.close(); S.previewWs = null; }

    const term = new Terminal({
        theme: { background: '#0c0c10', foreground: '#e8e8e8', cursor: '#8b5cf6' },
        fontSize: 14, fontFamily: "'JetBrains Mono','Fira Code',monospace",
        cursorBlink: true,
    });
    const fit = new FitAddon.FitAddon();
    term.loadAddon(fit);
    term.open(container);
    fit.fit();
    S.terminal = term;

    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    let url = `${proto}//${location.host}/ws/preview`;
    if (S.currentLesson) url += `?lesson=${S.currentLesson}`;
    const ws = new WebSocket(url);
    ws.binaryType = 'arraybuffer';
    S.previewWs = ws;

    ws.onopen = () => ws.send(JSON.stringify({ resize: { cols: term.cols, rows: term.rows } }));
    ws.onmessage = e => term.write(e.data instanceof ArrayBuffer ? new Uint8Array(e.data) : e.data);
    ws.onclose = () => term.write('\r\n\x1b[2m[disconnected]\x1b[0m\r\n');
    term.onData(d => { if (ws.readyState === 1) ws.send(d); });
    term.onResize(({ cols, rows }) => { if (ws.readyState === 1) ws.send(JSON.stringify({ resize: { cols, rows } })); });
    window.addEventListener('resize', () => fit.fit());
}

// ── Rail nav click handler ─────────────────────────────
document.querySelectorAll('.rail-btn[data-view]').forEach(btn => {
    btn.addEventListener('click', () => {
        if (btn.disabled) return;
        navigate(btn.dataset.view);
    });
});

// ── Header buttons ─────────────────────────────────────
document.getElementById('btn-validate').addEventListener('click', validateCourse);
document.getElementById('btn-preview-toggle').addEventListener('click', () => navigate('preview'));
document.getElementById('btn-preview-refresh').addEventListener('click', initPreview);

// ── Contextual Help Panel ──────────────────────────────
const HELP = {
    welcome: {
        title: 'Getting Started',
        body: `<h3>Welcome to LearnLocal Studio</h3>
<p>This is your course authoring environment. Here's how to get started:</p>
<p><strong>New Course</strong> — Creates a blank course in your workspace directory. Pick a name and language, then start adding lessons and exercises.</p>
<p><strong>Open Course</strong> — Load an existing course to edit. Any course directory with a <code>course.yaml</code> works.</p>
<h3>Course Structure</h3>
<pre>course.yaml          ← Course metadata
lessons/
  01-intro/
    lesson.yaml      ← Lesson config
    content.md       ← What students read
    exercises/
      01-hello.yaml  ← Exercise definition</pre>
<h3>Keyboard Shortcuts</h3>
<p><code>?</code> — Toggle this help panel</p>
<p><code>Ctrl+S</code> — Force save</p>`
    },
    course: {
        title: 'Course Overview',
        body: `<h3>Course Metadata</h3>
<p>Edit the course name, version, author, and description directly. Changes save automatically.</p>
<p>Use <strong>semantic versioning</strong> (e.g. 1.0.0). Progress is keyed to the major version — bumping from 1.x to 2.x resets student progress.</p>
<h3>Lessons</h3>
<p>Drag lessons by the grip handle to reorder. Click a lesson to edit its content and exercises.</p>
<p>Aim for <strong>6-8 exercises per lesson</strong>. Each lesson should teach one coherent concept.</p>`
    },
    lesson: {
        title: 'Lesson Editor',
        body: `<h3>Lesson Content</h3>
<p>The content tab is the markdown that students read before starting exercises. Use H2 headers (<code>##</code>) to split into sections for progressive reveal.</p>
<h3>Exercises</h3>
<p>Click an exercise to edit it. Drag to reorder. Mix exercise types for variety:</p>
<p>• <strong>write</strong> — Student writes code from scratch</p>
<p>• <strong>fix</strong> — Student fixes broken code</p>
<p>• <strong>fill-blank</strong> — Student fills in missing parts</p>
<p>• <strong>command</strong> — Student runs shell commands</p>`
    },
    exercise: {
        title: 'Exercise Editor',
        body: `<h3>Tabs</h3>
<p><strong>Prompt</strong> — What the student sees. Be specific about expected output.</p>
<p><strong>Solution</strong> — The correct answer. Run it to capture output.</p>
<p><strong>Starter</strong> — The code students begin with. Should compile but not pass.</p>
<p><strong>Validate</strong> — How to check if the student's code is correct.</p>
<p><strong>Hints</strong> — Progressive hints. 3 is ideal: conceptual → specific → near-answer.</p>
<p><strong>Explanation</strong> — Teaching moment shown after passing.</p>
<h3>Validation Methods</h3>
<p>• <strong>output</strong> — Exact string match against stdout</p>
<p>• <strong>regex</strong> — Pattern match against stdout</p>
<p>• <strong>compile-only</strong> — Just needs to compile</p>
<p>• <strong>state</strong> — Check filesystem (files exist, content, permissions)</p>
<h3>Pro Tips</h3>
<p>Use <strong>Run Solution</strong> to capture output, then <strong>Use as expected output</strong> to auto-fill validation.</p>`
    },
};

function toggleHelp() {
    const panel = document.getElementById('help-panel');
    panel.classList.toggle('open');
    if (panel.classList.contains('open')) updateHelpContent();
}

function updateHelpContent() {
    const help = HELP[S.currentView] || HELP.welcome;
    document.getElementById('help-content').innerHTML = `<h3>${help.title}</h3>${help.body}`;
}

// ── AI Chat Panel ──────────────────────────────────────
const aiMessages = [];

function toggleAI() {
    const panel = document.getElementById('ai-panel');
    panel.classList.toggle('open');
    if (panel.classList.contains('open') && aiMessages.length === 0) {
        addAiMessage('assistant', "Hi! I'm your course authoring assistant. I can help with:\n\n• Writing exercise prompts and solutions\n• Suggesting hints and explanations\n• Reviewing exercise quality\n• Structuring lessons and courses\n• Generating starter code\n\nWhat are you working on?");
        document.getElementById('ai-input').focus();
    }
}

function addAiMessage(role, content) {
    aiMessages.push({ role, content });
    renderAiMessages();
}

function renderAiMessages() {
    const container = document.getElementById('ai-messages');
    container.innerHTML = aiMessages.map(m => `
        <div class="ai-msg">
            <div class="ai-role ${m.role}">${m.role === 'user' ? 'You' : 'AI Assistant'}</div>
            <div class="ai-body ${m.role === 'user' ? 'user-msg' : ''}">${esc(m.content)}</div>
        </div>
    `).join('');
    container.scrollTop = container.scrollHeight;
}

async function sendAiMessage() {
    const input = document.getElementById('ai-input');
    const text = input.value.trim();
    if (!text) return;

    input.value = '';
    addAiMessage('user', text);

    // Add thinking indicator
    const thinkingIdx = aiMessages.length;
    aiMessages.push({ role: 'assistant', content: 'Thinking...' });
    renderAiMessages();

    try {
        // Build context about what the user is currently editing
        let context = '';
        if (S.currentView === 'exercise' && S.exerciseData) {
            context = `The author is editing exercise "${S.exerciseData.title}" (${S.exerciseData.type}) in lesson "${S.currentLesson}".`;
        } else if (S.currentView === 'lesson') {
            context = `The author is working on lesson "${S.currentLesson}".`;
        }

        const chatMessages = aiMessages
            .filter(m => m.content !== 'Thinking...')
            .map(m => ({ role: m.role, content: m.content }));

        const resp = await api('POST', '/ai/chat', { messages: chatMessages, context });

        // Extract response
        const reply = resp.choices?.[0]?.message?.content || 'No response from AI';
        aiMessages[thinkingIdx] = { role: 'assistant', content: reply };
        renderAiMessages();
    } catch (e) {
        aiMessages[thinkingIdx] = { role: 'assistant', content: `Error: ${e.message}\n\nMake sure Ollama is running on localhost:11434` };
        renderAiMessages();
    }
}

// ── Onboarding ─────────────────────────────────────────
// Each step targets a real DOM element and positions the tooltip next to it
const ONBOARDING_STEPS = [
    {
        target: null, // centered, no spotlight
        arrow: 'no-arrow',
        title: 'Welcome to LearnLocal Studio',
        body: `<p>Let's take a quick tour of the interface. Click <strong>Next</strong> to follow along, or <strong>Skip</strong> to jump right in.</p>`,
        position: 'center',
    },
    {
        target: '#rail',
        arrow: 'arrow-left',
        title: 'Navigation Rail',
        body: `<p>This rail shows your place in the hierarchy. Click to jump between levels:</p>
<p>📚 <strong>Course</strong> — overview &amp; lesson list</p>
<p>📝 <strong>Lesson</strong> — content &amp; exercises</p>
<p>⚡ <strong>Exercise</strong> — tabbed editor</p>
<p>🖥 <strong>Preview</strong> — at the bottom, opens a live terminal</p>`,
        position: 'right',
    },
    {
        target: '#breadcrumb',
        arrow: 'arrow-top',
        title: 'Breadcrumb',
        body: `<p>Shows where you are. Click any part to navigate back up the hierarchy.</p>`,
        position: 'below',
    },
    {
        target: '#rail-preview',
        arrow: 'arrow-left',
        title: 'Terminal Preview',
        body: `<p>Opens a <strong>live terminal</strong> running the real student TUI. See exactly what learners experience as you author.</p>`,
        position: 'right',
    },
    {
        target: '#btn-help',
        arrow: 'arrow-top',
        title: 'Contextual Help',
        body: `<p>Opens a help panel with docs relevant to what you're currently doing. Changes as you navigate.</p>`,
        position: 'below',
    },
    {
        target: '#btn-ai',
        arrow: 'arrow-top',
        title: 'AI Assistant',
        body: `<p>Chat with a local AI to help draft exercises, suggest hints, review quality, or brainstorm lesson structure.</p>
<p>Uses your local <strong>Ollama</strong> — nothing leaves your machine.</p>`,
        position: 'below',
    },
    {
        target: null,
        arrow: 'no-arrow',
        title: 'You\'re all set!',
        body: `<p>Everything <strong>auto-saves</strong> as you type. Use <strong>Validate</strong> to check course quality. Every change is tracked in the audit trail.</p><p>Happy authoring!</p>`,
        position: 'center',
    },
];

let onboardingStep = 0;
let spotlightEl = null;

function showOnboarding() {
    onboardingStep = 0;
    document.getElementById('onboarding').style.display = '';
    document.getElementById('onboarding-scrim').style.display = '';
    renderOnboardingStep();
}

function renderOnboardingStep() {
    const step = ONBOARDING_STEPS[onboardingStep];
    const card = document.getElementById('onboarding-card');
    const content = document.getElementById('onboarding-content');

    // Clear previous spotlight
    if (spotlightEl) { spotlightEl.classList.remove('onboarding-spotlight'); spotlightEl = null; }

    // Set content
    content.innerHTML = `<h2>${step.title}</h2>${step.body}`;
    document.getElementById('onboarding-step').textContent = `${onboardingStep + 1} / ${ONBOARDING_STEPS.length}`;

    // Set arrow direction
    card.className = `onboarding-card ${step.arrow}`;

    // Position card
    if (step.position === 'center' || !step.target) {
        card.style.left = '50%';
        card.style.top = '50%';
        card.style.transform = 'translate(-50%, -50%)';
        card.style.right = '';
        card.style.bottom = '';
    } else {
        card.style.transform = '';
        const el = document.querySelector(step.target);
        if (el) {
            el.classList.add('onboarding-spotlight');
            spotlightEl = el;
            const rect = el.getBoundingClientRect();

            if (step.position === 'right') {
                card.style.left = (rect.right + 16) + 'px';
                card.style.top = rect.top + 'px';
                card.style.right = '';
                card.style.bottom = '';
            } else if (step.position === 'below') {
                card.style.left = Math.max(8, rect.left - 20) + 'px';
                card.style.top = (rect.bottom + 12) + 'px';
                card.style.right = '';
                card.style.bottom = '';
                // Keep within viewport
                const cardWidth = 360;
                if (rect.left + cardWidth > window.innerWidth) {
                    card.style.left = '';
                    card.style.right = '12px';
                }
            }
        }
    }
}

function nextOnboarding() {
    onboardingStep++;
    if (onboardingStep >= ONBOARDING_STEPS.length) {
        endOnboarding();
    } else {
        renderOnboardingStep();
    }
}

function endOnboarding() {
    if (spotlightEl) { spotlightEl.classList.remove('onboarding-spotlight'); spotlightEl = null; }
    document.getElementById('onboarding').style.display = 'none';
    document.getElementById('onboarding-scrim').style.display = 'none';
    localStorage.setItem('learnlocal-onboarded', '1');
}

function checkOnboarding() {
    if (!localStorage.getItem('learnlocal-onboarded')) {
        setTimeout(showOnboarding, 500); // slight delay so UI renders first
    }
}

// ── Init ───────────────────────────────────────────────
(async () => {
    try {
        const status = await api('GET', '/workspace/status');
        if (status.has_course) {
            S.course = await api('GET', '/course');
            document.title = `${S.course.name} — LearnLocal Studio`;
            navigate('course');
        } else {
            document.title = 'LearnLocal Studio';
            navigate('welcome');
        }
    } catch (e) {
        toast('Failed to initialize: ' + e.message, 'error');
        navigate('welcome');
    }
    checkOnboarding();
})();
