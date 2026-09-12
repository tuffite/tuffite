const output = document.querySelector("#history-state");
const update = () => { output.textContent = `History length: ${history.length}`; };
document.querySelector("#history").onclick = () => {
  history.pushState({}, "", `?step=${history.length}`);
  update();
};
addEventListener("popstate", update);
update();
