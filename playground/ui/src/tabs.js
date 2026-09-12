const tabs = [...document.querySelectorAll("[role='tab'][data-tab]")];
const panels = [...document.querySelectorAll("[role='tabpanel'][data-panel]")];

function activate(name, focus = false) {
  for (const tab of tabs) {
    const selected = tab.dataset.tab === name;
    tab.setAttribute("aria-selected", String(selected));
    tab.tabIndex = selected ? 0 : -1;
    if (selected && focus) tab.focus();
  }
  for (const panel of panels) panel.hidden = panel.dataset.panel !== name;
}

for (const [index, tab] of tabs.entries()) {
  tab.addEventListener("click", () => activate(tab.dataset.tab));
  tab.addEventListener("keydown", (event) => {
    if (!new Set(["ArrowLeft", "ArrowRight", "Home", "End"]).has(event.key)) {
      return;
    }
    event.preventDefault();
    const next = event.key === "Home"
      ? 0
      : event.key === "End"
        ? tabs.length - 1
        : (index + (event.key === "ArrowRight" ? 1 : -1) + tabs.length) % tabs.length;
    activate(tabs[next].dataset.tab, true);
  });
}
