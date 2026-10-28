const page = new URL(location.href).searchParams.get("page") === "b" ? "B" : "A";
document.title = `History ${page}`;
document.querySelector("strong").textContent = document.title;
