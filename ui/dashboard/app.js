/**
 * Polymera OS — Kernel Dashboard Application
 * Real-time visualization of kernel internals with simulated data feed.
 */

// ─── Simulated Kernel State ───
const kernel = {
  ticks: 0, startTime: Date.now(), syscalls: 0, ipcRouted: 0, cryptoOps: 0,
  processes: [], nextPid: 2, ipc: { sent: 0, received: 0, queued: 0, bytes: 0, channels: 0 },
  memory: { kernel: 32, services: 64, user: 96, free: 64, pages: 12288, faults: 42, compressed: 18, huge: 8 },
  crypto: { dilSigns: 0, dilVerifies: 0, kyberEncaps: 0, kyberDecaps: 0, blake3Hashes: 0 },
  history: { processes: [], syscalls: [], ipc: [], memory: [], crypto: [], ticks: [] },
  log: []
};

function aiMetrics() {
  return window.POLYMERA_AI_METRICS || {
    ai_log_summaries_total: 0,
    ai_plans_generated_total: 0,
    ai_ltm_events_total: 0,
    ai_browser_summaries_total: 0,
    ai_capability_denials_total: 0,
    updated_at_unix: 0,
    last_summary: null,
  };
}

const SERVICES = [
  { name: 'kernel', priority: 'KERNEL', state: 'RUNNING', cpu: 2.1, mem: 32768 },
  { name: 'init', priority: 'HIGH', state: 'RUNNING', cpu: 0.3, mem: 8192 },
  { name: 'polybus', priority: 'HIGH', state: 'RUNNING', cpu: 1.8, mem: 16384 },
  { name: 'keyvault', priority: 'HIGH', state: 'RUNNING', cpu: 0.9, mem: 12288 },
  { name: 'ngfs', priority: 'NORMAL', state: 'RUNNING', cpu: 3.2, mem: 24576 },
  { name: 'polynet', priority: 'NORMAL', state: 'RUNNING', cpu: 4.1, mem: 20480 },
  { name: 'polyaudio', priority: 'NORMAL', state: 'READY', cpu: 0.1, mem: 8192 },
  { name: 'compositor', priority: 'NORMAL', state: 'RUNNING', cpu: 5.7, mem: 32768 },
  { name: 'attestation', priority: 'NORMAL', state: 'RUNNING', cpu: 0.4, mem: 4096 },
  { name: 'ai_runtime', priority: 'LOW', state: 'RUNNING', cpu: 8.3, mem: 65536 },
];

// Initialize processes
SERVICES.forEach((s, i) => {
  kernel.processes.push({
    pid: i, name: s.name, state: s.state, priority: s.priority,
    cpu: s.cpu, mem: s.mem, ipcSent: 0, syscalls: 0,
  });
});
kernel.nextPid = SERVICES.length;

// ─── Log Helper ───
function klog(level, msg) {
  const ts = ((Date.now() - kernel.startTime) / 1000).toFixed(3);
  const entry = `[${ts}s] [${level}] ${msg}`;
  kernel.log.push({ level, text: entry });
  if (kernel.log.length > 200) kernel.log.shift();
}

klog('INFO', 'Polymera OS Kernel Dashboard initialized');
klog('INFO', 'PQC Crypto: Dilithium + Kyber + BLAKE3 operational');
klog('INFO', `Spawned ${SERVICES.length} system services`);

// ─── Sparkline Drawing ───
function drawSparkline(containerId, data, color = '#06b6d4') {
  const el = document.getElementById(containerId);
  if (!el) return;
  const w = 60, h = 30;
  el.innerHTML = '';
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', `0 0 ${w} ${h}`);
  svg.setAttribute('width', w);
  svg.setAttribute('height', h);
  if (data.length < 2) { el.appendChild(svg); return; }
  const max = Math.max(...data, 1);
  const pts = data.map((v, i) => `${(i / (data.length - 1)) * w},${h - (v / max) * (h - 4) - 2}`).join(' ');
  const polyline = document.createElementNS('http://www.w3.org/2000/svg', 'polyline');
  polyline.setAttribute('points', pts);
  polyline.setAttribute('fill', 'none');
  polyline.setAttribute('stroke', color);
  polyline.setAttribute('stroke-width', '1.5');
  polyline.setAttribute('stroke-linecap', 'round');
  polyline.setAttribute('stroke-linejoin', 'round');
  svg.appendChild(polyline);
  el.appendChild(svg);
}

// ─── SLO Gauge Drawing ───
function drawGauge(canvas, value, color = '#22c55e') {
  const ctx = canvas.getContext('2d');
  const s = 2, w = canvas.width, h = canvas.height, cx = w/2, cy = h/2, r = (Math.min(w,h)/2) - 8;
  ctx.setTransform(1,0,0,1,0,0);
  ctx.clearRect(0, 0, w, h);
  // Background arc
  ctx.beginPath(); ctx.arc(cx, cy, r, 0.75*Math.PI, 2.25*Math.PI);
  ctx.strokeStyle = 'rgba(255,255,255,0.06)'; ctx.lineWidth = 8; ctx.lineCap = 'round'; ctx.stroke();
  // Value arc
  const angle = 0.75*Math.PI + (value/100) * 1.5*Math.PI;
  ctx.beginPath(); ctx.arc(cx, cy, r, 0.75*Math.PI, angle);
  ctx.strokeStyle = color; ctx.lineWidth = 8; ctx.lineCap = 'round'; ctx.stroke();
  // Center text
  ctx.fillStyle = '#f1f5f9'; ctx.font = '600 18px Inter'; ctx.textAlign = 'center';
  ctx.fillText(`${Math.round(value)}%`, cx, cy + 6);
}

// ─── IPC Flow Canvas ───
function drawIpcFlow() {
  const canvas = document.getElementById('ipc-canvas');
  if (!canvas) return;
  const ctx = canvas.getContext('2d');
  const w = canvas.width = canvas.parentElement.clientWidth - 20;
  const h = canvas.height = 250;
  ctx.clearRect(0, 0, w, h);

  const nodes = kernel.processes.filter(p => p.state === 'RUNNING').slice(0, 8);
  if (nodes.length < 2) return;
  const cx = w / 2, cy = h / 2, radius = Math.min(w, h) * 0.35;

  // Draw nodes in a circle
  const positions = nodes.map((n, i) => {
    const angle = (i / nodes.length) * 2 * Math.PI - Math.PI / 2;
    return { x: cx + Math.cos(angle) * radius, y: cy + Math.sin(angle) * radius, name: n.name };
  });

  // Draw edges (IPC channels)
  const t = kernel.ticks * 0.02;
  for (let i = 0; i < positions.length; i++) {
    for (let j = i + 1; j < positions.length; j++) {
      if (Math.random() > 0.5) continue;
      const a = positions[i], b = positions[j];
      const grad = ctx.createLinearGradient(a.x, a.y, b.x, b.y);
      grad.addColorStop(0, 'rgba(6,182,212,0.3)');
      grad.addColorStop(1, 'rgba(168,85,247,0.3)');
      ctx.beginPath(); ctx.moveTo(a.x, a.y); ctx.lineTo(b.x, b.y);
      ctx.strokeStyle = grad; ctx.lineWidth = 1; ctx.stroke();

      // Animated dot traveling along edge
      const progress = (Math.sin(t + i * 0.7 + j * 1.3) + 1) / 2;
      const dx = a.x + (b.x - a.x) * progress;
      const dy = a.y + (b.y - a.y) * progress;
      ctx.beginPath(); ctx.arc(dx, dy, 2.5, 0, 2 * Math.PI);
      ctx.fillStyle = '#06b6d4'; ctx.fill();
    }
  }

  // Draw node circles
  positions.forEach((p, i) => {
    ctx.beginPath(); ctx.arc(p.x, p.y, 16, 0, 2 * Math.PI);
    ctx.fillStyle = 'rgba(17,24,39,0.9)'; ctx.fill();
    ctx.strokeStyle = i < 2 ? '#a855f7' : '#06b6d4'; ctx.lineWidth = 2; ctx.stroke();
    ctx.fillStyle = '#f1f5f9'; ctx.font = '500 9px Inter'; ctx.textAlign = 'center';
    ctx.fillText(p.name, p.x, p.y + 3);
  });
}

// ─── Supervisor Tree ───
function renderSupervisorTree() {
  const el = document.getElementById('supervisor-tree');
  if (!el) return;
  const tree = [
    { name: 'root-supervisor', strategy: 'one-for-one', state: 'running', children: [
      { name: 'network-super', strategy: 'one-for-one', state: 'running', children: [
        { name: 'polynet', state: 'running' }, { name: 'polybus', state: 'running' },
      ]},
      { name: 'storage-super', strategy: 'one-for-all', state: 'running', children: [
        { name: 'ngfs', state: 'running' }, { name: 'keyvault', state: 'running' },
      ]},
      { name: 'ui-super', strategy: 'rest-for-one', state: 'running', children: [
        { name: 'compositor', state: 'running' }, { name: 'polyaudio', state: kernel.ticks % 100 < 5 ? 'restarting' : 'running' },
      ]},
      { name: 'ai_runtime', state: 'running' },
    ]},
  ];
  el.innerHTML = renderNode(tree[0], 0);
}

function renderNode(node, depth) {
  const indent = depth > 0 ? ' style="margin-left:' + (depth * 20) + 'px"' : '';
  let html = `<div class="sup-node"${indent}>
    <span class="sup-dot ${node.state}"></span>
    <span class="sup-name">${node.name}</span>
    ${node.strategy ? `<span class="sup-strategy">${node.strategy}</span>` : ''}
  </div>`;
  if (node.children) {
    html += '<div class="sup-indent">';
    node.children.forEach(c => { html += renderNode(c, depth + 1); });
    html += '</div>';
  }
  return html;
}

// ─── Process Table ───
let selectedPid = null;
function renderProcessTable() {
  const tbody = document.getElementById('process-tbody');
  if (!tbody) return;
  tbody.innerHTML = kernel.processes.map(p => {
    const stateClass = `state-${p.state.toLowerCase()}`;
    const memStr = p.mem >= 1024 ? (p.mem / 1024).toFixed(1) + ' MB' : p.mem + ' KB';
    const sel = p.pid === selectedPid ? ' selected' : '';
    return `<tr class="${sel}" data-pid="${p.pid}" onclick="selectProcess(${p.pid})">
      <td>${p.pid}</td><td>${p.name}</td>
      <td><span class="state-badge ${stateClass}">${p.state}</span></td>
      <td>${p.priority}</td><td>${p.cpu.toFixed(1)}%</td>
      <td>${memStr}</td><td>${p.ipcSent}</td><td>${p.syscalls}</td></tr>`;
  }).join('');
}

function selectProcess(pid) { selectedPid = selectedPid === pid ? null : pid; renderProcessTable(); }

function spawnProcess() {
  const names = ['worker', 'daemon', 'handler', 'scanner', 'watcher', 'analyzer', 'bridge', 'proxy'];
  const name = names[Math.floor(Math.random() * names.length)] + '_' + kernel.nextPid;
  kernel.processes.push({
    pid: kernel.nextPid++, name, state: 'RUNNING', priority: 'NORMAL',
    cpu: Math.random() * 5, mem: Math.floor(Math.random() * 16384) + 1024,
    ipcSent: 0, syscalls: 0,
  });
  klog('INFO', `Spawned process "${name}" (PID ${kernel.nextPid - 1})`);
}

function killProcess() {
  if (selectedPid === null || selectedPid < 2) return;
  const idx = kernel.processes.findIndex(p => p.pid === selectedPid);
  if (idx !== -1) {
    klog('WARN', `Terminated process "${kernel.processes[idx].name}" (PID ${selectedPid})`);
    kernel.processes.splice(idx, 1);
    selectedPid = null;
  }
}

// ─── Log Panel ───
function renderLog() {
  const content = document.getElementById('log-content');
  const count = document.getElementById('log-count');
  if (!content) return;
  content.innerHTML = kernel.log.map(e => {
    const cls = e.level === 'WARN' ? 'log-entry-warn' : e.level === 'ERROR' ? 'log-entry-error' : e.level === 'OK' ? 'log-entry-success' : 'log-entry-info';
    return `<span class="${cls}">${e.text}</span>`;
  }).join('\n');
  count.textContent = kernel.log.length + ' entries';
  content.parentElement.scrollTop = content.parentElement.scrollHeight;
}

// ─── Uptime ───
function updateUptime() {
  const s = Math.floor((Date.now() - kernel.startTime) / 1000);
  const h = String(Math.floor(s / 3600)).padStart(2, '0');
  const m = String(Math.floor((s % 3600) / 60)).padStart(2, '0');
  const sec = String(s % 60).padStart(2, '0');
  document.getElementById('uptime').textContent = `${h}:${m}:${sec}`;
}

// ─── Main Update Loop ───
function update() {
  kernel.ticks++;
  // Simulate syscalls & IPC
  const newSyscalls = Math.floor(Math.random() * 50) + 10;
  const newIpc = Math.floor(Math.random() * 30) + 5;
  kernel.syscalls += newSyscalls;
  kernel.ipcRouted += newIpc;
  kernel.ipc.sent += newIpc;
  kernel.ipc.received += Math.floor(newIpc * 0.9);
  kernel.ipc.queued = Math.floor(Math.random() * 20);
  kernel.ipc.bytes += newIpc * (Math.floor(Math.random() * 2048) + 256);
  kernel.ipc.channels = kernel.processes.filter(p => p.state === 'RUNNING').length;

  // Simulate crypto ops
  const newCrypto = Math.floor(Math.random() * 10);
  kernel.cryptoOps += newCrypto;
  kernel.crypto.dilSigns += Math.floor(Math.random() * 3);
  kernel.crypto.dilVerifies += Math.floor(Math.random() * 5);
  kernel.crypto.kyberEncaps += Math.floor(Math.random() * 2);
  kernel.crypto.kyberDecaps += Math.floor(Math.random() * 2);
  kernel.crypto.blake3Hashes += Math.floor(Math.random() * 50) + 10;

  // Simulate process CPU jitter
  kernel.processes.forEach(p => {
    if (p.state === 'RUNNING') {
      p.cpu = Math.max(0, p.cpu + (Math.random() - 0.5) * 2);
      p.ipcSent += Math.floor(Math.random() * 3);
      p.syscalls += Math.floor(Math.random() * 5);
    }
  });

  // Memory jitter
  kernel.memory.user = Math.max(60, Math.min(140, kernel.memory.user + (Math.random() - 0.48) * 3));
  kernel.memory.free = 256 - kernel.memory.kernel - kernel.memory.services - kernel.memory.user;
  kernel.memory.pages += Math.floor(Math.random() * 10);
  kernel.memory.faults += Math.random() > 0.9 ? 1 : 0;

  // Push history
  const pushH = (arr, val) => { arr.push(val); if (arr.length > 30) arr.shift(); };
  pushH(kernel.history.processes, kernel.processes.length);
  const metrics = aiMetrics();
  const aiLogSummaries = metrics.ai_log_summaries_total || 0;
  const aiPlansGenerated = metrics.ai_plans_generated_total || 0;
  const aiBrowserSummaries = metrics.ai_browser_summaries_total || 0;
  const aiCapabilityDenials = metrics.ai_capability_denials_total || 0;
  pushH(kernel.history.syscalls, aiBrowserSummaries);
  pushH(kernel.history.ipc, aiCapabilityDenials);
  pushH(kernel.history.memory, Math.round(kernel.memory.user));
  pushH(kernel.history.crypto, aiPlansGenerated);
  pushH(kernel.history.ticks, aiLogSummaries);

  // Periodic log entries
  if (kernel.ticks % 20 === 0) klog('INFO', `Tick ${kernel.ticks}: ${kernel.processes.length} processes, ${newSyscalls} syscalls/tick`);
  if (kernel.ticks % 50 === 0) klog('OK', `Security check passed — all capabilities valid`);
  if (kernel.ticks % 70 === 0) klog('INFO', `PQC: ${kernel.crypto.dilSigns} signatures, ${kernel.crypto.kyberEncaps} encapsulations`);

  // ── Update DOM ──
  document.getElementById('metric-processes').textContent = kernel.processes.length;
  document.getElementById('metric-syscalls').textContent = aiBrowserSummaries.toLocaleString();
  document.getElementById('metric-ipc').textContent = aiCapabilityDenials.toLocaleString();
  document.getElementById('metric-memory').textContent = Math.round(kernel.memory.user) + ' MB';
  document.getElementById('metric-crypto').textContent = aiPlansGenerated.toLocaleString();
  document.getElementById('metric-ticks').textContent = aiLogSummaries.toLocaleString();

  drawSparkline('spark-processes', kernel.history.processes, '#22c55e');
  drawSparkline('spark-syscalls', kernel.history.syscalls, '#06b6d4');
  drawSparkline('spark-ipc', kernel.history.ipc, '#a855f7');
  drawSparkline('spark-memory', kernel.history.memory, '#f59e0b');
  drawSparkline('spark-crypto', kernel.history.crypto, '#ec4899');
  drawSparkline('spark-ticks', kernel.history.ticks, '#3b82f6');

  renderProcessTable();
  drawIpcFlow();

  document.getElementById('ipc-channel-count').textContent = kernel.ipc.channels + ' channels';
  document.getElementById('ipc-sent').textContent = kernel.ipc.sent.toLocaleString();
  document.getElementById('ipc-received').textContent = kernel.ipc.received.toLocaleString();
  document.getElementById('ipc-queued').textContent = kernel.ipc.queued;
  document.getElementById('ipc-throughput').textContent = Math.round(kernel.ipc.bytes / 1024 / Math.max(1, (Date.now() - kernel.startTime) / 1000)) + ' KB/s';

  // Memory map widths
  const total = kernel.memory.kernel + kernel.memory.services + kernel.memory.user + kernel.memory.free;
  document.getElementById('mem-kernel-region').style.flex = kernel.memory.kernel / total * 10;
  document.getElementById('mem-services-region').style.flex = kernel.memory.services / total * 10;
  document.getElementById('mem-user-region').style.flex = kernel.memory.user / total * 10;
  document.getElementById('mem-free-region').style.flex = kernel.memory.free / total * 10;
  document.getElementById('mem-free-region').querySelector('.mem-size').textContent = Math.round(kernel.memory.free) + ' MB';
  document.getElementById('mem-user-region').querySelector('.mem-size').textContent = Math.round(kernel.memory.user) + ' MB';
  document.getElementById('mem-pages').textContent = kernel.memory.pages.toLocaleString();
  document.getElementById('mem-faults').textContent = kernel.memory.faults;

  // Crypto
  document.getElementById('dil-signs').textContent = kernel.crypto.dilSigns;
  document.getElementById('dil-verifies').textContent = kernel.crypto.dilVerifies;
  document.getElementById('kyber-encaps').textContent = kernel.crypto.kyberEncaps;
  document.getElementById('kyber-decaps').textContent = kernel.crypto.kyberDecaps;
  document.getElementById('blake3-hashes').textContent = kernel.crypto.blake3Hashes.toLocaleString();
  document.getElementById('blake3-throughput').textContent = Math.round(kernel.crypto.blake3Hashes * 0.064) + ' MB/s';

  renderSupervisorTree();

  // SLO gauges
  const gauges = document.querySelectorAll('.slo-gauge');
  const sloValues = [97, 99, 95, 98, 96, 92];
  const sloColors = ['#22c55e','#22c55e','#22c55e','#22c55e','#22c55e','#f59e0b'];
  gauges.forEach((g, i) => {
    const c = g.querySelector('canvas');
    if (c) {
      const jitter = (Math.random() - 0.5) * 4;
      drawGauge(c, Math.min(100, Math.max(80, sloValues[i] + jitter)), sloColors[i]);
    }
  });

  updateUptime();
  renderLog();
}

// ─── Event Listeners ───
document.getElementById('btn-spawn').addEventListener('click', spawnProcess);
document.getElementById('btn-kill').addEventListener('click', killProcess);
document.getElementById('log-toggle').addEventListener('click', () => {
  document.getElementById('kernel-log').classList.toggle('collapsed');
});

// Start collapsed
document.getElementById('kernel-log').classList.add('collapsed');

// ─── Start ───
update();
setInterval(update, 1000);
