
// SERVER DATA STRUCTURE (Each server has its own files)
const servers = [
    {
        name: "Server1",
        files: {
            "Document1.pdf": "This is the content of Document1.pdf...",
            "Image.png": "This file is an image, preview not available.",
            "Presentation.pptx": "Presentation about our latest project...",
            "Spreadsheet.xlsx": "Spreadsheet data showing company profits..."
        }
    },
    {
        name: "Server2",
        files: {
            "Report.docx": "Annual financial report...",
            "Photo.jpg": "Vacation photo...",
            "Notes.txt": "Meeting notes..."
        }
    },
    {
        name: "Server3",
        files: {
            "Music.mp3": "Favorite song...",
            "Video.mp4": "A short movie...",
            "Slides.pptx": "Presentation for class..."
        }
    }
];


// TRACK CURRENT SERVER INDEX
let currentServerIndex = 0;

// FUNCTION TO NAVIGATE SERVERS
function navigateServer(direction) {
    currentServerIndex += direction;

    // Ensure index stays within bounds
    if (currentServerIndex < 0) {
        currentServerIndex = servers.length - 1; // Loop to last server
    } else if (currentServerIndex >= servers.length) {
        currentServerIndex = 0; // Loop to first server
    }

    // Update UI
    updateServerDisplay();
}



// FUNCTION TO UPDATE UI WHEN SWITCHING SERVERS
function updateServerDisplay() {
    const currentServer = servers[currentServerIndex];

    // Update Server Name
    document.getElementById("current-server").textContent = currentServer.name;

    // Update File List
    updateFileList(currentServer.files);
}


// FUNCTION TO OPEN POP-UP WITH FILE CONTENT
function openPopup(fileName) {
    const popup = document.getElementById("file-popup");
    const popupTitle = document.getElementById("popup-title");
    const popupFileContent = document.getElementById("popup-file-content");

    // Get the current server
    const currentServer = servers[currentServerIndex];

    // Get file content from the selected server
    const fileContent = currentServer.files[fileName] || "No content available.";

    // Set file title and content
    popupTitle.textContent = fileName;
    popupFileContent.textContent = fileContent;

    // Show pop-up
    popup.style.display = "flex";
}

// Close the Pop-Up
function closePopup() {
    document.getElementById("file-popup").style.display = "none";
}


// INITIALIZE UI WITH FIRST SERVER
document.addEventListener("DOMContentLoaded", updateServerDisplay);



// FUNCTION TO UPDATE FILE LIST BASED ON SERVER
function updateFileList(files) {
    const fileListTable = document.querySelector(".file-list tbody");
    fileListTable.innerHTML = ""; // Clear existing content

    // Populate new file list
    for (const [fileName, fileContent] of Object.entries(files)) {
        const row = document.createElement("tr");
        row.innerHTML = `
            <td>${fileName}</td>
            <td style="text-align:right;">${servers[currentServerIndex].name}</td>
        `;
        row.addEventListener("click", () => openPopup(fileName, fileContent));
        fileListTable.appendChild(row);
    }
}




function populate_files_and_images(){

}





// Attach Click Event to Each File Row
document.addEventListener("DOMContentLoaded", function () {
    document.querySelectorAll(".file-list tr").forEach(row => {
        row.addEventListener("click", function () {
            const fileName = this.cells[0].textContent.trim(); // Get file name from row
            openPopup(fileName);
        });
    });
});





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
