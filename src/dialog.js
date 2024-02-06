const dialog = document.getElementById("info-dialog");
const showButton = document.getElementById("show-button");
const closeButton = document.getElementById("close-button");

showButton.addEventListener("click", () => {
  dialog.showModal();
});

closeButton.addEventListener("click", () => {
  dialog.close();
});
