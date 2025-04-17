export function setupDropzone(dropzone, fileInput, fileNameDisplay) {
	dropzone.addEventListener("click", () => fileInput.click());
	dropzone.addEventListener("dragover", (e) => {
		e.preventDefault();
		dropzone.classList.add("border-green-400");
	});
	dropzone.addEventListener("dragleave", () => {
		dropzone.classList.remove("border-green-400");
	});
	dropzone.addEventListener("drop", (e) => {
		e.preventDefault();
		const file = e.dataTransfer.files[0];
		if (file) {
			fileInput.files = e.dataTransfer.files;
			fileNameDisplay.textContent = `File: ${file.name}`;
		}
		dropzone.classList.remove("border-green-400");
	});
	fileInput.addEventListener("change", () => {
		if (fileInput.files.length > 0) {
			fileNameDisplay.textContent = `File: ${fileInput.files[0].name}`;
		} else {
			fileNameDisplay.textContent = "";
		}
	});
}

export function showUrl(link, urlsection, urlinput, st, sb) {
	urlsection.classList.remove("hidden");
	urlinput.value = link;
	st.textContent = "Upload Successful.";
	sb.disabled = false;
	turnstile.reset();
}

export function resetForm(
	st,
	sb,
	urlinput,
	urlsection,
	btnText = "Upload Failed.",
) {
	urlinput.value = "";
	urlsection.classList.add("hidden");
	st.textContent = btnText;
	sb.disabled = false;
	turnstile.reset();
}
