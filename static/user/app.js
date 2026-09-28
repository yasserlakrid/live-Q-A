const state = {
  currentScreen: 'welcome',
  managers: [],
  questions: [],
  visitor: null,
  selectedManager: null,
  chosenQuestion: null,
  ws: null,
};

const screens = {
  welcome: document.getElementById('screen-welcome'),
  name: document.getElementById('screen-name'),
  manager: document.getElementById('screen-manager'),
  questions: document.getElementById('screen-questions'),
  waiting: document.getElementById('screen-waiting'),
};

const managerList = document.getElementById('manager-list');
const questionList = document.getElementById('question-list');
const confirmModal = document.getElementById('confirm-modal');
const confirmQuestionText = document.getElementById('confirm-question-text');
const toast = document.getElementById('toast');
const nameError = document.getElementById('name-error');

function showScreen(name) {
  state.currentScreen = name;
  Object.entries(screens).forEach(([key, element]) => {
    element.classList.toggle('active', key === name);
  });
}

function showToast(message) {
  toast.textContent = message;
  toast.classList.remove('hidden');
  clearTimeout(showToast.timeoutId);
  showToast.timeoutId = setTimeout(() => toast.classList.add('hidden'), 2500);
}

function setManagerStatusDot(status) {
  const classes = { Available: 'available', Busy: 'busy', Offline: 'offline' };
  const type = classes[status] || 'available';
  return `<span class="status-dot ${type === 'busy' ? 'busy' : type === 'offline' ? 'offline' : ''}"></span>`;
}

async function loadManagers() {
  const response = await fetch('/api/managers');
  if (!response.ok) {
    showToast('Could not load managers.');
    return;
  }

  state.managers = await response.json();
  renderManagerCards();
}

async function loadQuestionDefinitions() {
  const response = await fetch('/api/questions/definitions');
  if (!response.ok) {
    showToast('Could not load question topics.');
    return;
  }

  state.questions = await response.json();
  renderQuestions();
}

function renderManagerCards() {
  managerList.innerHTML = '';

  state.managers.forEach((manager) => {
    const card = document.createElement('button');
    const isOffline = manager.status === 'Offline';
    card.type = 'button';
    card.className = `manager-card ${state.selectedManager?.id === manager.id ? 'selected' : ''} ${isOffline ? 'disabled' : ''}`;
    card.disabled = isOffline;
    card.innerHTML = `
      <div class="manager-name">${manager.name}</div>
      <div class="manager-status">
        ${setManagerStatusDot(manager.status)}
        <span>${manager.status}</span>
      </div>
      <div class="manager-bio">${manager.bio}</div>
    `;

    card.addEventListener('click', async () => {
      if (isOffline) {
        showToast('This manager is currently unavailable.');
        return;
      }

      state.selectedManager = manager;
      renderManagerCards();

      const name = document.getElementById('visitor-name').value.trim();
      if (!name) {
        showScreen('name');
        nameError.textContent = 'Please enter your name to continue.';
        return;
      }

      const response = await fetch('/api/visitors', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name, manager_id: state.selectedManager.id }),
      });

      if (!response.ok) {
        const payload = await response.json().catch(() => ({}));
        showToast(payload.error || 'Could not create your visitor session.');
        return;
      }

      state.visitor = await response.json();
      nameError.textContent = '';
      openVisitorSocket();
      showScreen('questions');
    });

    managerList.appendChild(card);
  });
}

function renderQuestions() {
  questionList.innerHTML = '';

  state.questions.forEach((question) => {
    const button = document.createElement('button');
    button.type = 'button';
    button.className = 'question-bubble';
    button.textContent = question.text;
    button.addEventListener('click', () => {
      state.chosenQuestion = question;
      confirmQuestionText.textContent = `Ask ${state.selectedManager?.name} this question?`;
      confirmModal.classList.remove('hidden');
    });
    questionList.appendChild(button);
  });
}

async function createVisitor() {
  const name = document.getElementById('visitor-name').value.trim();
  if (!name) {
    nameError.textContent = 'Please enter your name.';
    return;
  }

  if (!state.selectedManager) {
    nameError.textContent = 'Please select a manager first.';
    return;
  }

  const response = await fetch('/api/visitors', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, manager_id: state.selectedManager.id }),
  });

  if (!response.ok) {
    const payload = await response.json().catch(() => ({}));
    nameError.textContent = payload.error || 'Could not create your visitor session.';
    return;
  }

  const visitor = await response.json();
  state.visitor = visitor;
  nameError.textContent = '';
  openVisitorSocket();
  showScreen('questions');
}

function openVisitorSocket() {
  if (state.ws) {
    state.ws.close();
  }

  const socket = new WebSocket(`${window.location.origin.replace(/^http/, 'ws')}/ws?role=visitor&id=${state.visitor.id}`);
  state.ws = socket;

  socket.addEventListener('message', (event) => {
    try {
      const payload = JSON.parse(event.data);
      if (payload.type === 'question_status_changed') {
        updateWaitingStatus(payload.data);
      }
    } catch (error) {
      console.warn('Malformed websocket message', error);
    }
  });

  socket.addEventListener('close', () => {
    if (!state.visitor) return;
    setTimeout(() => {
      openVisitorSocket();
    }, 1500);
  });
}

function updateWaitingStatus(data) {
  const waitingStatus = document.getElementById('waiting-status');
  const waitingManagerName = document.getElementById('waiting-manager-name');
  const waitingManagerMeta = document.getElementById('waiting-manager-meta');
  const waitingQuestionText = document.getElementById('waiting-question-text');
  const waitingVisitorName = document.getElementById('waiting-visitor-name');

  waitingStatus.textContent = data.status === 'Waiting'
    ? 'Waiting for manager'
    : data.status === 'InProgress'
      ? 'In progress'
      : 'Answered';

  waitingManagerName.textContent = state.selectedManager?.name || data.manager_name || 'The manager';
  waitingManagerMeta.textContent = state.selectedManager?.name || data.manager_name || 'Unknown';
  waitingQuestionText.textContent = data.question || 'Your question';
  waitingVisitorName.textContent = state.visitor?.name || 'Visitor';

  if (state.currentScreen !== 'waiting') {
    showScreen('waiting');
  }
}

async function submitQuestion() {
  if (!state.visitor || !state.selectedManager || !state.chosenQuestion) {
    showToast('Please choose a valid question.');
    return;
  }

  const response = await fetch('/api/questions', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      visitor_id: state.visitor.id,
      manager_id: state.selectedManager.id,
      question: state.chosenQuestion.text,
    }),
  });

  if (!response.ok) {
    const payload = await response.json().catch(() => ({}));
    showToast(payload.error || "Couldn't send your question. Please try again.");
    return;
  }

  const question = await response.json();
  state.chosenQuestion = question;
  confirmModal.classList.add('hidden');

  const waitingQuestionText = document.getElementById('waiting-question-text');
  waitingQuestionText.textContent = question.question;
  document.getElementById('waiting-manager-name').textContent = state.selectedManager.name;
  document.getElementById('waiting-visitor-name').textContent = state.visitor.name;
  document.getElementById('waiting-manager-meta').textContent = state.selectedManager.name;
  document.getElementById('waiting-status').textContent = 'Waiting for manager';
  showScreen('waiting');
}

function bindEvents() {
  document.getElementById('start-button').addEventListener('click', () => {
    showScreen('name');
  });

  document.getElementById('continue-button').addEventListener('click', () => {
    const name = document.getElementById('visitor-name').value.trim();
    if (!name) {
      nameError.textContent = 'Please enter your name.';
      return;
    }
    nameError.textContent = '';
    showScreen('manager');
  });

  document.getElementById('change-manager-button').addEventListener('click', () => {
    state.selectedManager = null;
    showScreen('manager');
  });

  document.getElementById('ask-another-button').addEventListener('click', () => {
    state.chosenQuestion = null;
    showScreen('questions');
  });

  document.getElementById('confirm-cancel').addEventListener('click', () => {
    confirmModal.classList.add('hidden');
  });

  document.getElementById('confirm-submit').addEventListener('click', submitQuestion);

  document.getElementById('visitor-name').addEventListener('keydown', (event) => {
    if (event.key === 'Enter') createVisitor();
  });
}

async function init() {
  bindEvents();
  await loadManagers();
  await loadQuestionDefinitions();
  showScreen('welcome');
}

init();
