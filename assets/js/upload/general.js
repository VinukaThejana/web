import { setupDropzone } from "./upload-utils.js";

const dropzone = document.getElementById("dropzone");
const fileInput = document.getElementById("file-input");
const fileNameDisplay = document.getElementById("file-name");

setupDropzone(dropzone, fileInput, fileNameDisplay);
