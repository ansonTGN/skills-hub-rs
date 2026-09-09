const invoke = window.__TAURI__.core.invoke;

const state = {
  dashboard: null,
  recycle: [],
  search: [],
  scan: [],
  activeView: "dashboard",
};

const titles = {
  dashboard: ["CONTROL PLANE", "Dashboard"],
  skills: ["LIBRARY", "Skills gestionados"],
  tools: ["ADAPTERS", "Herramientas y agentes"],
  install: ["INGEST", "Instalar Skills"],
  search: ["DISCOVERY", "Buscar en skills.sh"],
  scan: ["MIGRATION", "Descubrir Skills existentes"],
  recycle: ["RECOVERY", "Papelera recuperable"],
};

function esc(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}

function toast(message, isError = false) {
  const el = document.getElementById("toast");
  el.textContent = message;
  el.className = `toast show${isError ? " error" : ""}`;
  clearTimeout(toast.timer);
  toast.timer = setTimeout(() => (el.className = "toast"), 3600);
}

async function call(command, args = {}) {
  try {
    return await invoke(command, args);
  } catch (error) {
    toast(String(error), true);
    throw error;
  }
}

async function refresh() {
  state.dashboard = await call("dashboard");
  document.getElementById("hubRoot").textContent = state.dashboard.root;
  renderAll();
}

function setView(name) {
  state.activeView = name;
  document.querySelectorAll(".view").forEach(v => v.classList.remove("active"));
  document.getElementById(`view-${name}`).classList.add("active");
  document.querySelectorAll(".nav-item").forEach(btn => btn.classList.toggle("active", btn.dataset.view === name));
  document.getElementById("eyebrow").textContent = titles[name][0];
  document.getElementById("pageTitle").textContent = titles[name][1];
  if (name === "recycle") loadRecycle();
}

function statusBadge(enabled) {
  return enabled ? '<span class="badge ok">● enabled</span>' : '<span class="badge off">○ disabled</span>';
}

function renderDashboard() {
  const d = state.dashboard;
  const skills = d.skills;
  const tools = d.tools;
  const targets = skills.flatMap(s => s.targets);
  const detected = tools.filter(t => t.detected).length;
  const healthy = targets.filter(t => t.status === "ok").length;
  const recent = skills.slice(0, 7);
  const activeTools = tools.filter(t => t.detected).slice(0, 8);

  document.getElementById("view-dashboard").innerHTML = `
    <div class="stats">
      <div class="stat"><span class="label">Skills</span><span class="value">${skills.length}</span><div class="hint">biblioteca central</div></div>
      <div class="stat"><span class="label">Targets</span><span class="value">${targets.length}</span><div class="hint">${healthy} sincronizados OK</div></div>
      <div class="stat"><span class="label">Tools</span><span class="value">${tools.length}</span><div class="hint">${detected} detectados</div></div>
      <div class="stat"><span class="label">Storage</span><span class="value">SQLite</span><div class="hint mono">${esc(d.db)}</div></div>
    </div>
    <div class="grid-2">
      <div class="card">
        <div class="card-header"><div><h2>Skills gestionados</h2><p>Estado de la biblioteca y targets</p></div><button class="btn small ghost" data-jump="skills">Ver todos</button></div>
        ${recent.length ? `<div class="skill-list">${recent.map(skillRow).join("")}</div>` : empty("No hay Skills instalados", "Instala un Skill local o desde Git para comenzar.")}
      </div>
      <div class="card">
        <div class="card-header"><div><h2>Entorno detectado</h2><p>Adaptadores disponibles en este host</p></div></div>
        ${activeTools.length ? `<div class="kv">${activeTools.map(t => `<div><strong>${esc(t.label)}</strong><span>${esc(t.key)}</span></div>`).join("")}</div>` : empty("Sin tools detectados", "La detección se basa en los directorios conocidos.")}
      </div>
    </div>`;
}

function skillRow(view) {
  const s = view.skill;
  const tags = view.tags.map(t => `<span class="chip">${esc(t)}</span>`).join("");
  const targetInfo = view.targets.length ? `<span class="chip">${view.targets.length} target${view.targets.length === 1 ? "" : "s"}</span>` : "";
  return `<div class="skill-row">
    <div>
      <div class="skill-title">${statusBadge(s.enabled)}<strong>${esc(s.name)}</strong><span class="badge blue">${esc(s.source_type)}</span></div>
      <div class="skill-desc">${esc(s.description || "Sin descripción")}</div>
      <div class="chips">${tags}${targetInfo}</div>
    </div>
    <div class="row-actions">
      <button class="btn small" data-detail="${esc(s.name)}">Detalles</button>
      <button class="btn small ghost" data-update="${esc(s.name)}">↻</button>
      <button class="btn small ${s.enabled ? "ghost" : "primary"}" data-toggle="${esc(s.name)}" data-enabled="${s.enabled}">${s.enabled ? "Desactivar" : "Activar"}</button>
    </div>
  </div>`;
}

function renderSkills() {
  const skills = state.dashboard.skills;
  document.getElementById("view-skills").innerHTML = `
    <div class="card">
      <div class="card-header"><div><h2>Biblioteca central</h2><p>${esc(state.dashboard.root)}/skills</p></div><button class="btn small" id="updateAllBtn">↻ Actualizar todos</button></div>
      ${skills.length ? `<div class="skill-list">${skills.map(skillRow).join("")}</div>` : empty("Biblioteca vacía", "No hay Skills gestionados todavía.")}
    </div>`;
}

function renderTools() {
  const tools = state.dashboard.tools;
  const detected = tools.filter(t => t.detected);
  const other = tools.filter(t => !t.detected);
  const cards = arr => arr.map(t => `<div class="tool-card">
      <h3>${esc(t.label)}</h3>
      <p class="mono">${esc(t.global_dir)}</p>
      <div class="tool-meta"><span class="badge ${t.detected ? "ok" : "off"}">${t.detected ? "● detectado" : "○ no detectado"}</span><span class="badge">${t.force_copy ? "copy" : esc(t.mode)}</span></div>
    </div>`).join("");
  document.getElementById("view-tools").innerHTML = `
    <div class="card">
      <div class="card-header"><div><h2>Detectados (${detected.length})</h2><p>Herramientas localizadas en este equipo</p></div></div>
      <div class="tool-grid">${cards(detected)}</div>
    </div>
    <div class="card" style="margin-top:16px">
      <div class="card-header"><div><h2>Catálogo completo (${tools.length})</h2><p>Built-in y herramientas personalizadas</p></div></div>
      <div class="tool-grid">${cards(other)}</div>
    </div>
    <div class="card" style="margin-top:16px">
      <div class="card-header"><div><h2>Añadir herramienta personalizada</h2><p>Define rutas globales y opcionalmente de proyecto</p></div></div>
      ${customToolForm()}
    </div>`;
}

function customToolForm() {
  return `<form id="customToolForm" class="form-grid">
    <div class="field"><label>Key</label><input name="key" placeholder="mi_agente" required /></div>
    <div class="field"><label>Etiqueta</label><input name="label" placeholder="Mi agente" required /></div>
    <div class="field full"><label>Directorio global</label><input name="global_dir" placeholder="~/.mi-agente/skills" required /></div>
    <div class="field"><label>Directorio de proyecto</label><input name="project_dir" placeholder=".mi-agente/skills" /></div>
    <div class="field"><label>Directorio de detección</label><input name="detect_dir" placeholder="~/.mi-agente/skills" /></div>
    <div class="field"><label>Modo</label><select name="mode"><option>auto</option><option>symlink</option><option>copy</option><option>junction</option></select></div>
    <div class="field" style="align-self:end"><button class="btn primary" type="submit">Guardar herramienta</button></div>
  </form>`;
}

function renderInstall() {
  document.getElementById("view-install").innerHTML = `
    <div class="grid-2">
      <div class="card">
        <div class="card-header"><div><h2>Origen local</h2><p>Importa uno o varios directorios con SKILL.md</p></div></div>
        <form id="localInstallForm">
          <div class="field"><label>Ruta local</label><input name="path" placeholder="/ruta/al/skill" required /><span class="help">Puede ser un Skill directo o una carpeta con Skills anidados.</span></div>
          <div class="field" style="margin-top:12px"><label>Tags</label><input name="tags" placeholder="rust, seguridad, agentes" /></div>
          <div class="form-actions"><button class="btn primary" type="submit">Instalar local</button></div>
        </form>
      </div>
      <div class="card">
        <div class="card-header"><div><h2>Repositorio Git</h2><p>Clona, descubre SKILL.md e instala</p></div></div>
        <form id="gitInstallForm">
          <div class="field"><label>URL</label><input name="url" placeholder="https://github.com/org/repo.git" required /></div>
          <div class="form-grid" style="margin-top:12px"><div class="field"><label>Branch / tag</label><input name="revision" placeholder="main o v1.0.0" /></div><div class="field"><label>Subdirectorio</label><input name="subdir" placeholder="skills" /></div></div>
          <div class="field" style="margin-top:12px"><label>Tags</label><input name="tags" placeholder="github, rust" /></div>
          <div class="form-actions"><button class="btn primary" type="submit">Instalar desde Git</button></div>
        </form>
      </div>
    </div>`;
}

function renderSearch() {
  document.getElementById("view-search").innerHTML = `
    <div class="card">
      <div class="card-header"><div><h2>Índice público skills.sh</h2><p>Busca por nombre, tecnología o dominio</p></div></div>
      <form id="searchForm" class="searchbar"><input name="query" placeholder="rust security kubernetes..." required /><button class="btn primary">Buscar</button></form>
      <div id="searchResults">${state.search.length ? searchResults() : empty("Escribe una búsqueda", "Los resultados se consultan directamente en skills.sh.")}</div>
    </div>`;
}

function searchResults() {
  return `<div class="result-list">${state.search.map((r, i) => `<div class="result">
    <div><h3>${esc(r.name)}</h3><p>${esc(r.source)} · ${Number(r.installs).toLocaleString()} instalaciones</p></div>
    <button class="btn small primary" data-install-result="${i}">Instalar Git</button>
  </div>`).join("")}</div>`;
}

function renderScan() {
  document.getElementById("view-scan").innerHTML = `
    <div class="card">
      <div class="card-header"><div><h2>Descubrimiento local</h2><p>Busca SKILL.md en los directorios de tools configurados</p></div><button class="btn primary" id="scanBtn">Ejecutar scan</button></div>
      <div id="scanResults">${state.scan.length ? scanResults() : empty("Aún no se ha ejecutado el scan", "No modifica nada: solo descubre Skills existentes.")}</div>
    </div>`;
}

function scanResults() {
  if (!state.scan.length) return empty("Sin Skills detectados", "No se encontraron SKILL.md en las rutas configuradas.");
  return `<div class="table-wrap"><table><thead><tr><th>Skill</th><th>Tool</th><th>Ruta</th><th>Tipo</th></tr></thead><tbody>${state.scan.map(x => `<tr><td>${esc(x.name)}</td><td>${esc(x.tool || "-")}</td><td class="mono">${esc(x.path)}</td><td>${x.is_link ? '<span class="badge blue">link</span>' : '<span class="badge">directorio</span>'}</td></tr>`).join("")}</tbody></table></div>`;
}

async function loadRecycle() {
  state.recycle = await call("recycle_list");
  renderRecycle();
}

function renderRecycle() {
  const el = document.getElementById("view-recycle");
  if (!el) return;
  el.innerHTML = `<div class="card">
    <div class="card-header"><div><h2>Papelera local</h2><p>Recuperación de borrados durante 30 días</p></div><button class="btn small danger" id="purgeRecycleBtn">Purgar expirados</button></div>
    ${state.recycle.length ? `<div class="table-wrap"><table><thead><tr><th>Skill</th><th>Tags</th><th>Targets</th><th>Eliminado</th><th></th></tr></thead><tbody>${state.recycle.map(r => `<tr><td><strong>${esc(r.name)}</strong></td><td>${r.tags.map(t => `<span class="chip">${esc(t)}</span>`).join(" ")}</td><td>${r.target_count}</td><td>${new Date(r.deleted_at).toLocaleString()}</td><td><button class="btn small primary" data-restore="${esc(r.id)}">Restaurar</button></td></tr>`).join("")}</tbody></table></div>` : empty("Papelera vacía", "No hay Skills pendientes de recuperación.")}
  </div>`;
}

function empty(title, text) {
  return `<div class="empty"><strong>${esc(title)}</strong>${esc(text)}</div>`;
}

function renderAll() {
  renderDashboard();
  renderSkills();
  renderTools();
  renderInstall();
  renderSearch();
  renderScan();
  renderRecycle();
}

function getSkill(name) {
  return state.dashboard.skills.find(x => x.skill.name === name);
}

function openDetail(name) {
  const view = getSkill(name);
  if (!view) return;
  const s = view.skill;
  const toolOptions = state.dashboard.tools.map(t => `<option value="${esc(t.key)}">${esc(t.label)} (${esc(t.key)})</option>`).join("");
  const tags = view.tags.map(t => `<span class="chip">${esc(t)} <button class="close" style="font-size:11px" data-remove-tag="${esc(t)}" data-skill="${esc(s.name)}">×</button></span>`).join("");
  const targets = view.targets.length ? view.targets.map(t => `<div class="target"><div class="target-head"><div><strong>${esc(t.tool)}</strong> <span class="badge ${t.status === "ok" ? "ok" : t.status === "disabled" ? "off" : "error"}">${esc(t.status)}</span></div><button class="btn small danger" data-unsync="${esc(s.name)}" data-tool="${esc(t.tool)}" data-scope="${esc(t.scope)}" data-project="${esc(t.project_path || "")}">Unsync</button></div><p>${esc(t.target_path)}</p></div>`).join("") : empty("Sin targets", "Sincroniza este Skill con una herramienta.");

  document.getElementById("modalRoot").innerHTML = `<div class="modal-backdrop"><div class="modal">
    <div class="modal-head"><div><h2>${esc(s.name)}</h2><p>${esc(s.description || "Sin descripción")}</p></div><button class="close" data-close>×</button></div>
    <div class="kv">
      <div><strong>Origen</strong><span>${esc(s.source_type)} · ${esc(s.source_ref || "-")}</span></div>
      <div><strong>Central</strong><span class="mono">${esc(s.central_path)}</span></div>
      <div><strong>Estado</strong><span>${s.enabled ? "enabled" : "disabled"}</span></div>
    </div>
    <div style="margin-top:16px"><label>Tags</label><div class="chips" style="margin-top:7px">${tags || '<span class="muted">Sin tags</span>'}</div>
      <form id="tagForm" class="searchbar" style="margin-top:9px"><input name="tag" placeholder="nuevo-tag" required/><button class="btn small">Añadir tag</button></form>
    </div>
    <div style="margin-top:18px"><label>Targets</label>${targets}</div>
    <div class="card" style="margin-top:18px;box-shadow:none">
      <div class="card-header"><div><h2>Nueva sincronización</h2><p>Expone la copia central al tool seleccionado</p></div></div>
      <form id="syncForm" class="form-grid">
        <div class="field"><label>Tool</label><select name="tool">${toolOptions}</select></div>
        <div class="field"><label>Scope</label><select name="scope"><option value="global">global</option><option value="project">project</option></select></div>
        <div class="field"><label>Modo</label><select name="mode"><option>auto</option><option>symlink</option><option>copy</option><option>junction</option></select></div>
        <div class="field"><label>Proyecto (si scope=project)</label><input name="project" placeholder="/ruta/al/proyecto" /></div>
        <div class="field full"><label><input type="checkbox" name="overwrite" style="width:auto;margin-right:7px" /> permitir overwrite explícito</label></div>
        <div class="field full"><button class="btn primary" type="submit">Sincronizar</button></div>
      </form>
    </div>
    <div class="form-actions" style="justify-content:space-between;margin-top:18px"><button class="btn danger" data-delete="${esc(s.name)}">Mover a papelera</button><button class="btn ghost" data-close>Cerrar</button></div>
  </div></div>`;

  document.getElementById("tagForm").onsubmit = async e => {
    e.preventDefault();
    const tag = new FormData(e.target).get("tag");
    await call("add_tag", { skill: s.name, tag });
    await refresh(); openDetail(s.name); toast("Tag añadido");
  };
  document.getElementById("syncForm").onsubmit = async e => {
    e.preventDefault();
    const fd = new FormData(e.target);
    await call("sync_skill", {
      skill: s.name, tool: fd.get("tool"), scope: fd.get("scope"),
      project: fd.get("project") || null, mode: fd.get("mode"), overwrite: fd.get("overwrite") === "on"
    });
    await refresh(); openDetail(s.name); toast("Skill sincronizado");
  };
}

function tags(value) {
  return String(value || "").split(",").map(x => x.trim()).filter(Boolean);
}

document.addEventListener("click", async e => {
  const nav = e.target.closest("[data-view]"); if (nav) setView(nav.dataset.view);
  const jump = e.target.closest("[data-jump]"); if (jump) setView(jump.dataset.jump);
  if (e.target.closest("[data-close]") || e.target.classList.contains("modal-backdrop")) document.getElementById("modalRoot").innerHTML = "";

  const detail = e.target.closest("[data-detail]"); if (detail) openDetail(detail.dataset.detail);
  const update = e.target.closest("[data-update]"); if (update) { const status = await call("update_skill", { skill: update.dataset.update }); await refresh(); toast(`Update: ${status}`); }
  const toggle = e.target.closest("[data-toggle]"); if (toggle) { await call("set_enabled", { skill: toggle.dataset.toggle, enabled: toggle.dataset.enabled !== "true" }); await refresh(); toast("Estado actualizado"); }
  const del = e.target.closest("[data-delete]"); if (del && confirm(`¿Mover ${del.dataset.delete} a la papelera?`)) { await call("remove_skill", { skill: del.dataset.delete, permanent: false }); document.getElementById("modalRoot").innerHTML = ""; await refresh(); toast("Skill movido a la papelera"); }
  const restore = e.target.closest("[data-restore]"); if (restore) { await call("recycle_restore", { id: restore.dataset.restore }); await refresh(); await loadRecycle(); toast("Skill restaurado"); }
  const unsync = e.target.closest("[data-unsync]"); if (unsync) { await call("unsync_skill", { skill: unsync.dataset.unsync, tool: unsync.dataset.tool, scope: unsync.dataset.scope, project: unsync.dataset.project || null }); await refresh(); openDetail(unsync.dataset.unsync); toast("Target eliminado"); }
  const removeTag = e.target.closest("[data-remove-tag]"); if (removeTag) { await call("remove_tag", { skill: removeTag.dataset.skill, tag: removeTag.dataset.removeTag }); await refresh(); openDetail(removeTag.dataset.skill); }
  const installResult = e.target.closest("[data-install-result]"); if (installResult) { const r = state.search[Number(installResult.dataset.installResult)]; await call("install_git", { url: r.source_url, revision: null, subdir: null, tags: ["skills.sh"] }); await refresh(); toast(`${r.name} instalado`); setView("skills"); }
});

document.getElementById("refreshBtn").onclick = async () => { await refresh(); toast("Estado recargado"); };

document.addEventListener("submit", async e => {
  if (e.target.id === "localInstallForm") {
    e.preventDefault(); const fd = new FormData(e.target);
    await call("install_local", { path: fd.get("path"), tags: tags(fd.get("tags")) }); await refresh(); toast("Skill local instalado"); setView("skills");
  }
  if (e.target.id === "gitInstallForm") {
    e.preventDefault(); const fd = new FormData(e.target);
    await call("install_git", { url: fd.get("url"), revision: fd.get("revision") || null, subdir: fd.get("subdir") || null, tags: tags(fd.get("tags")) }); await refresh(); toast("Skill Git instalado"); setView("skills");
  }
  if (e.target.id === "searchForm") {
    e.preventDefault(); const q = new FormData(e.target).get("query");
    state.search = await call("search_online", { query: q, limit: 20 }); renderSearch();
  }
  if (e.target.id === "customToolForm") {
    e.preventDefault(); const fd = new FormData(e.target);
    await call("save_custom_tool", { key: fd.get("key"), label: fd.get("label"), globalDir: fd.get("global_dir"), projectDir: fd.get("project_dir") || null, detectDir: fd.get("detect_dir") || null, mode: fd.get("mode") }); await refresh(); toast("Herramienta personalizada guardada"); setView("tools");
  }
});

document.addEventListener("click", async e => {
  if (e.target.id === "updateAllBtn") { const result = await call("update_all"); await refresh(); toast(`${result.length} Skills procesados`); }
  if (e.target.id === "scanBtn") { state.scan = await call("scan_installed", { tool: null }); renderScan(); toast(`${state.scan.length} Skills detectados`); }
  if (e.target.id === "purgeRecycleBtn") { const n = await call("recycle_purge", { all: false }); await loadRecycle(); toast(`${n} entradas purgadas`); }
});

refresh().catch(error => {
  document.getElementById("view-dashboard").innerHTML = empty("No se pudo abrir el motor", String(error));
});
