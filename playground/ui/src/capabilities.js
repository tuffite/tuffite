const hosts = document.querySelector("#hosts");
const result = document.querySelector("#result");
const create = (suffix) => {
  const element = document.createElement("explorer");
  element.navigation = "allow";
  element.style.cssText = "display:block;flex:1;height:100%;border:1px solid #aaa";
  const url = new URL("playground://main/ui/dist/capability-sample.html");
  // Packaged URLs use separate, SOP-isolated hostnames.
  url.hostname = `demo.${suffix}`;
  element.src = url.href;
  hosts.append(element);
  return element;
};
const artifact = create("artifact");
const preview = create("preview");
window.capabilitiesDemo = { artifact, preview };
document.querySelector("#run").onclick = async () => {
  try {
    result.textContent = JSON.stringify(await Promise.all([artifact, preview].map(
      (element) => element.executeJavaScript("JSON.parse(document.body.dataset.capabilityReport || 'null')")
    )), null, 2);
  } catch (error) { result.textContent = error.message; }
};
