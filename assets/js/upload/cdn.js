import { handleUpload } from "./upload-handler.js";

const form = document.getElementById("upload-form");
const st = document.getElementById("upload-status");
const sb = document.getElementById("upload-button");
const urlsection = document.getElementById("url-section");
const urlinput = document.getElementById("url-input");

handleUpload({ form, st, sb, urlsection, urlinput, useCDN: true });
