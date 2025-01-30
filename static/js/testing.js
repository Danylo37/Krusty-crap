function searchGoogleDrive() {
    const searchInput = document.querySelector('.search-input').value.trim();
    if (searchInput) {
        alert(`Searching for: ${searchInput}`);
        // You can replace this with an actual search function
    }
}

function handleSearchKeyPress(event) {
    if (event.key === "Enter") {
        searchGoogleDrive();
    }
}

function filterGoogleDrive() {
    alert("Filter function clicked! (Placeholder for filtering logic)");
    // You can replace this with an actual filter function
}

function logOut() {
    alert("Logging out...");
    window.location.reload(); // Simulating logout by refreshing
}
