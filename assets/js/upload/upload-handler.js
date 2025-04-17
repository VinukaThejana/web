import { showUrl, resetForm } from "./upload-utils.js";

export async function handleUpload({
	form,
	st,
	sb,
	urlsection,
	urlinput,
	useCDN = false,
}) {
	form.addEventListener("submit", async (e) => {
		e.preventDefault();

		const formData = new FormData(form);
		const captcha = formData.get("cf-turnstile-response");
		const file = formData.get("file");
		const path = formData.get("path");
		const password = formData.get("password");

		if (!captcha) {
			st.textContent = "Are you a robot ?";
			return;
		}
		if (!file || !(file instanceof File)) {
			st.textContent = "Please select a file to upload.";
			return;
		}
		if (!path) {
			st.textContent = "Please select a path.";
			return;
		}
		if (!password) {
			st.textContent = "Please enter a password.";
			return;
		}

		sb.disabled = true;
		st.textContent = "Uploading ... ";

		const response = await fetch("/api/upload/storage", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({
				path,
				password,
				"cf-turnstile-response": captcha,
			}),
		});

		const payload = await response.json();

		if (!response.ok || !payload.url) {
			resetForm(
				st,
				sb,
				urlinput,
				urlsection,
				payload.status ?? "Upload failed.",
			);
			return;
		}

		const s3 = await fetch(payload.url, {
			method: "PUT",
			headers: {
				"Content-Type": file.type,
				"Content-Length": file.size.toString(),
			},
			body: file,
		});
		if (!s3.ok) {
			resetForm(st, sb, urlinput, urlsection);
			return;
		}

		const s3_url = `https://blob.vinuka.dev/${path}`;

		if (!useCDN) {
			showUrl(s3_url, urlsection, urlinput, st, sb);
			return;
		}

		const cdn = await fetch("/api/upload/cdn", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({ path, password, url: s3_url }),
		});

		const content = await cdn.json();

		if (!cdn.ok || !content.url) {
			resetForm(
				st,
				sb,
				urlinput,
				urlsection,
				content.status ?? "Upload failed.",
			);
			return;
		}

		showUrl(content.url, urlsection, urlinput, st, sb);
	});
}
