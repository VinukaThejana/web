// Set current Unix timestamp on "Now" button click
document.getElementById("set-current-date").addEventListener("click", () => {
	const now = Math.floor(Date.now() / 1000); // Unix timestamp in seconds
	document.getElementById("date").value = now;
});

const updateCounter = (field, counterId, max) => {
	const value = document.getElementById(field).value;
	const count = value.length;
	const counterEl = document.getElementById(counterId);
	counterEl.textContent = `${count}/${max}`;
	counterEl.classList.toggle("text-red-500", count > max);
	counterEl.classList.toggle("text-gray-400", count <= max);
};

updateCounter("summary", "summary-counter", 160);
updateCounter("content", "content-counter", 100000);

document.getElementById("summary").addEventListener("input", () => {
	updateCounter("summary", "summary-counter", 160);
});
document.getElementById("content").addEventListener("input", () => {
	updateCounter("content", "content-counter", 100000);
});
