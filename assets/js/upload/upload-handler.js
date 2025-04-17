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

		if (!captcha || !file || !(file instanceof File) || !path || !password) {
			st.textContent = !captcha
				? "Are you a robot ?"
				: !file
					? "Please select a file to upload."
					: !path
						? "Please select a path."
						: "Please enter a password.";
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

		const s3_url = `https://blob.vinuka.dev/${path}`;

		if (!useCDN) {
			// Upload with progress using XMLHttpRequest
			const xhr = new XMLHttpRequest();
			xhr.open("PUT", payload.url, true);
			xhr.setRequestHeader("Content-Type", file.type);
			xhr.setRequestHeader("Content-Length", file.size.toString());

			xhr.upload.onprogress = (event) => {
				if (event.lengthComputable) {
					const percent = Math.round((event.loaded / event.total) * 100);
					st.textContent = `Uploading: ${percent}%`;
				}
			};

			xhr.onload = () => {
				if (xhr.status >= 200 && xhr.status < 300) {
					showUrl(s3_url, urlsection, urlinput, st, sb);
				} else {
					resetForm(st, sb, urlinput, urlsection);
				}
			};

			xhr.onerror = () => {
				resetForm(st, sb, urlinput, urlsection);
			};

			xhr.send(file);
			return;
		}

		// If using CDN (2-step process)
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

		const cdn = await fetch("/api/upload/cdn", {
			method: "POST",
			headers: { "Content-Type": "application/json" },
			body: JSON.stringify({
				path,
				password,
				url: s3_url,
			}),
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
