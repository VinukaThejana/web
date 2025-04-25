document.getElementById("long_url").focus();

const updateCounter = (field, counterId, max) => {
	const value = document.getElementById(field).value;
	const count = value.length;
	const counterEl = document.getElementById(counterId);
	counterEl.textContent = `${count}/${max}`;
	counterEl.classList.toggle("text-red-500", count > max);
	counterEl.classList.toggle("text-gray-400", count <= max);
};

updateCounter("description", "description-counter", 160);

document.getElementById("description").addEventListener("input", () => {
	updateCounter("description", "description-counter", 160);
});
