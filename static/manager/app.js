const state = {
  managers: [],
  selectedManagerId: 'yasser',
  questions: [],
  ws: null,
};

const managerPicker = document.getElementById('manager-picker');
const queueList = document.getElementById('queue-list');
const welcomeTitle = document.getElementById('welcome-title');
const connectionStatus = document.getElementById('connection-status');
const toast = document.getElementById('toast');

function showToast(message) {
  toast.textContent = message;
  toast.classList.remove('hidden');
  clearTimeout(showToast.timeoutId);
  showToast.timeoutId = setTimeout(() => toast.classList.add('hidden'), 2200);
}

function setConnectionStatus(connected) {
  connectionStatus.textContent = connected ? 'Connected' : 'Reconnecting…';
  connectionStatus.classList.toggle('disconnected', !connected);
}

function getSelectedManager() {
  return state.managers.find((manager) => manager.id === state.selectedManagerId) || state.managers[0];
}

function renderManagerPicker() {
  managerPicker.innerHTML = '';
  state.managers.forEach((manager) => {
    const option = document.createElement('option');
    option.value = manager.id;
    option.textContent = `${manager.name} (${manager.status})`;
    managerPicker.appendChild(option);
  });
  managerPicker.value = state.selectedManagerId;
  syncManagerHeader();
}

function syncManagerHeader() {
  const manager = getSelectedManager();
  if (!manager) return;
  welcomeTitle.textContent = `Welcome, ${manager.name}`;
  updateStatusButtons(manager.status);
}

function updateStatusButtons(status) {
  document.querySelectorAll('.status-toggle').forEach((button) => {
    button.classList.toggle('active', button.dataset.status === status);
  });
}

async function loadManagers() {
  const response = await fetch('/api/managers');
  if (!response.ok) {
    showToast('Could not load managers.');
    return;
  }

  state.managers = await response.json();
  if (!state.selectedManagerId && state.managers.length) {
    state.selectedManagerId = state.managers[0].id;
  }
  renderManagerPicker();
  refreshQueue();
}

async function updateManagerStatus(status) {
  const manager = getSelectedManager();
  if (!manager) return;

  const response = await fetch(`/api/managers/${manager.id}/status`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ status }),
  });

  if (!response.ok) {
    const payload = await response.json().catch(() => ({}));
    showToast(payload.error || 'Could not update manager status.');
    return;
  }

  const updated = await response.json();
  const index = state.managers.findIndex((item) => item.id === updated.id);
  if (index >= 0) state.managers[index] = updated;
  renderManagerPicker();
  refreshQueue();
}

function filterQuestions() {
  return state.questions.filter((question) => question.manager_id === state.selectedManagerId);
}

function renderQueue() {
  const questions = filterQuestions();
  queueList.innerHTML = '';

  if (!questions.length) {
    queueList.innerHTML = '<div class="empty-state">No questions for this manager yet.</div>';
    updateStats();
    return;
  }

  questions
    .sort((a, b) => new Date(b.created_at) - new Date(a.created_at))
    .forEach((question) => {
      const card = document.createElement('article');
      card.className = 'question-card';
      card.innerHTML = `
        <h3>${question.visitor_name}</h3>
        <p>"${question.question}"</p>
        <div class="question-meta">
          <span>${question.created_at}</span>
          <span>${question.status}</span>
        </div>
        <div class="question-actions">
          <button class="action-button" type="button" data-id="${question.id}" data-status="Waiting">Start</button>
          <button class="action-button answered" type="button" data-id="${question.id}" data-status="Answered">Mark Answered</button>
        </div>
      `;

      card.querySelector('[data-status="Waiting"]').addEventListener('click', () => updateQuestionStatus(question.id, 'InProgress'));
      card.querySelector('[data-status="Answered"]').addEventListener('click', () => updateQuestionStatus(question.id, 'Answered'));
      queueList.appendChild(card);
    });

  updateStats();
}

function updateStats() {
  const questions = filterQuestions();
  document.getElementById('stat-waiting').textContent = questions.filter((q) => q.status === 'Waiting').length;
  document.getElementById('stat-progress').textContent = questions.filter((q) => q.status === 'InProgress').length;
  document.getElementById('stat-answered').textContent = questions.filter((q) => q.status === 'Answered').length;
  document.getElementById('stat-total').textContent = questions.length;
}

async function refreshQueue() {
  const response = await fetch('/api/questions');
  if (!response.ok) return;
  state.questions = await response.json();
  renderQueue();
}

async function updateQuestionStatus(questionId, nextStatus) {
  const response = await fetch(`/api/questions/${questionId}/status`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ status: nextStatus }),
  });

  if (!response.ok) {
    const payload = await response.json().catch(() => ({}));
    showToast(payload.error || 'Could not update question.');
    return;
  }

  const updated = await response.json();
  const index = state.questions.findIndex((question) => question.id === updated.id);
  if (index >= 0) state.questions[index] = updated;
  refreshQueue();
}

function openSocket() {
  if (state.ws) state.ws.close();

  const manager = getSelectedManager();
  if (!manager) return;

  const socketUrl = `${window.location.origin.replace(/^http/, 'ws')}/ws?role=manager&id=${manager.id}`;
  const socket = new WebSocket(socketUrl);
  state.ws = socket;

  socket.addEventListener('open', () => {
    setConnectionStatus(true);
    showToast('Connected to live queue');
  });

  socket.addEventListener('message', (event) => {
    try {
      const payload = JSON.parse(event.data);
      if (payload.type === 'new_question') {
        const question = payload.data;
        const existing = state.questions.find((item) => item.id === question.id);
        if (!existing) state.questions.push({
          id: question.id,
          visitor_name: question.visitor_name,
          question: question.question,
          created_at: question.created_at,
          status: question.status,
          manager_id: question.manager_id,
          visitor_id: question.visitor_id || '',
        });
        renderQueue();
      }

      if (payload.type === 'question_status_changed') {
        const incoming = payload.data;
        const index = state.questions.findIndex((item) => item.id === incoming.id);
        if (index >= 0) {
          state.questions[index].status = incoming.status;
          state.questions[index].question = incoming.question;
        } else {
          state.questions.push({
            id: incoming.id,
            visitor_name: incoming.visitor_name || 'Visitor',
            question: incoming.question,
            created_at: incoming.created_at,
            status: incoming.status,
            manager_id: state.selectedManagerId,
            visitor_id: '',
          });
        }
        renderQueue();
      }
    } catch (error) {
      console.warn('Malformed websocket message', error);
    }
  });

  socket.addEventListener('close', () => {
    setConnectionStatus(false);
    setTimeout(() => {
      if (document.visibilityState === 'visible') openSocket();
    }, 1500);
  });
}

function bindEvents() {
  managerPicker.addEventListener('change', (event) => {
    state.selectedManagerId = event.target.value;
    renderManagerPicker();
    openSocket();
  });

  document.querySelectorAll('.status-toggle').forEach((button) => {
    button.addEventListener('click', () => {
      updateManagerStatus(button.dataset.status);
    });
  });

  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible') {
      refreshQueue();
      openSocket();
    }
  });
}

async function init() {
  bindEvents();
  await loadManagers();
  await refreshQueue();
  openSocket();
}

init();
