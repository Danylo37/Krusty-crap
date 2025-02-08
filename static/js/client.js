



/*


   USER VIEW
   -> WHATSAPP: SEND MESSAGE, CREATING AND SWITCHING BETWEEN CHATS


*/


function openChat(chatName, clientId) {
    // Update the chat header with the selected chat name.
    document.getElementById('chat-header').textContent = chatName;


    // Remove the update dot from the clicked chat item and mark it as active.
    const chatItem = document.querySelector(`.chat-item[data-id="${clientId}"]`);
    if (chatItem) {
        console.log("updatingChat")
        const chatMessages = document.getElementById('chat-messages');
        if (chatMessages.style.background != 'url("content_objects/whatsapp_bg.jpg")'){
            chatMessages.style.background = 'url("content_objects/whatsapp_bg.jpg")';
            chatMessages.style.backgroundSize = 'cover';
            chatMessages.style.backgroundPosition = 'center';
        }

        // Hide the update dot.
        const updateDot = chatItem.querySelector('.update-dot');
        if (updateDot) {
            updateDot.style.display = 'none';
        }
        // Remove 'active' class from any other chat items.
        document.querySelectorAll('.chat-item').forEach(item => item.classList.remove('active'));
        // Mark this chat as active.
        chatItem.classList.add('active');


        // Check if there is stored chat history for this chat item.
        if (chatItem.dataset.history) {
            const history = JSON.parse(chatItem.dataset.history);
            updateChatWindow(history);
        } else {
            // Optionally display a placeholder if no history exists.
            chatMessages.innerHTML = `<div style="text-align:center; padding:20px;">No messages yet.</div>`;
        }
    }
}






// Phonebooks STORED
const phonebooks = {
};


/*
const phonebooks = {
   "0": ["0", "2", "5"],
   "1": ["1", "3", "4"],
   "2": ["7", "10", "15"],
};
*/


function createNewChat() {
    const chatPopup = document.getElementById('chat-popup');
    const receiverList = document.getElementById('receiver-list');


    const listBoxes = [];


    // Create list boxes dynamically for each server
    Object.keys(phonebooks).forEach((serverId) => {


        if (document.getElementById(`server-container-${serverId}`)) {
            // Container already exists, so skip creating a new one.
            return;
        }


        // Server container
        const serverContainer = document.createElement('div');
        serverContainer.id = `server-container-${serverId}`;
        serverContainer.style = "margin-bottom: 15px;";




        //Container name + search
        const labelContainer = document.createElement('div');
        labelContainer.style = "display:flex; align-items: center; justify-content: space-between; margin-bottom: 5px;";


        // Server label
        const serverLabel = document.createElement('label');
        serverLabel.textContent = `Phonebook id: ${serverId}: `;
        serverLabel.style = "font-weight: bold;";
        labelContainer.appendChild(serverLabel);



        // Create an image element for search.png and insert it next to the label
        const searchImg = document.createElement('img');
        searchImg.src = "content_objects/reload.png"; // Adjust path if needed
        searchImg.alt = "Search";
        searchImg.className = "comm-ui-reload-button";
        searchImg.id = `reload-${serverId}`;
        searchImg.onclick = function(event) {
            this.style.animation = "spin 1s linear infinite";
            askListRegisteredClientsToServer(currentClientId , serverId);
        };
        labelContainer.appendChild(searchImg);


        //adding both to flex div
        serverContainer.appendChild(labelContainer);




        // Server list box
        const listBox = document.createElement('select');
        listBox.className = 'server-list-box';
        listBox.id = `server-list-box-${serverId}`;
        listBox.style = "margin-left: 10px; padding: 5px;";


        // Default "Not Choosed" option
        const defaultOption = document.createElement('option');
        defaultOption.value = '';
        defaultOption.textContent = 'Not Choosed';
        listBox.appendChild(defaultOption);


        // Gather existing chats (assuming each chat item’s text is the receiver id)
        const existingChats = Array.from(document.getElementById('chat-list').children)
            .map(item => item.dataset.id);

        console.log(existingChats)
        if (Array.isArray(phonebooks[serverId])) {
            phonebooks[serverId].forEach((receiver) => {
                if (existingChats.includes(receiver.toString())) {
                    // Skip if a chat with this receiver already exists
                    return;
                }
                const option = document.createElement('option');
                option.value = receiver;
                option.textContent = receiver;
                listBox.appendChild(option);
            });
        } else {
            console.error("Expected an array for phonebooks[" + serverId + "], but got", phonebooks[serverId]);
        }


        // Add change event to handle exclusivity among list boxes
        listBox.addEventListener('change', () => {
            listBoxes.forEach((box) => {
                if (box !== listBox) {
                    box.value = ''; // Reset others
                }
            });
        });


        listBoxes.push(listBox);


        // Append the list box to the server container
        serverContainer.appendChild(listBox);


        // Append the server container to the receiver list
        receiverList.appendChild(serverContainer);
    });


    chatPopup.style.display = 'flex'; // Show the popup
}


function confirmSelection() {
    const listBoxes = document.querySelectorAll('.server-list-box');
    let selectedReceiver = '';


    // Get the selected receiver (if any) from the list boxes
    listBoxes.forEach((box) => {
        if (box.value) {
            selectedReceiver = box.value;
        }
    });


    if (!selectedReceiver) {
        alert('Please select a receiver.');
        return;
    }


    // Create a new chat item in the chat list (sidebar)
    const chatList = document.getElementById('chat-list');
    const chatItem = document.createElement('div');
    chatItem.className = 'chat-item';
    chatItem.dataset.id = selectedReceiver; // assign the data-id attribute


    // Include both the chat name and the update dot
    chatItem.innerHTML = `<span class="chat-name">${selectedReceiver}</span>
                         <span class="update-dot" style="display: none;"></span>`;


    // Set the onclick handler to open the chat and remove the notification dot
    chatItem.onclick = () => {
        openChat(selectedReceiver, selectedReceiver);
        // Remove the update dot when the chat is selected:
        const dot = chatItem.querySelector('.update-dot');
        if (dot) dot.style.display = 'none';
    };


    chatList.appendChild(chatItem);


    // Optionally, open the new chat immediately
    openChat(selectedReceiver, selectedReceiver);


    // Close the chat pop-up
    closeChatPopup();
}


function closeChatPopup() {
    const chatPopup = document.getElementById('chat-popup');
    chatPopup.style.display = 'none'; // Hide the popup
}






























/*


   Requesting and Updating


*/




// Requesting
function askListRegisteredClientsToServer(clientId, serverId){
    // CHen/Chen/CHEN function to retrieve List registered clients
    if (ws.readyState === WebSocket.OPEN) {
        // Construct the message with the actual values of whichClient and whichServer
        const message = {
            WsAskListRegisteredClientsToServer: {
                client_id: clientId.toString(), // Ensure u64 is sent as a string
                server_id: serverId.toString(), // Ensure u64 is sent as a string
            }
        };
        ws.send(JSON.stringify(message));
        console.log('Sent:', message);
    } else {
        console.error('WebSocket is not open. Unable to send update command.');
    }
}




function sendMessage() {
    console.log("message sent html")
    const messageInput = document.getElementById('message-input');
    const chatMessages = document.getElementById('chat-messages');
    const messageText = messageInput.value.trim();
    const activeChatItem = document.querySelector('.chat-item.active');

    if (!messageText) {
        alert("Insert something in input");
        return
    } // Do nothing if message is empty


    sendMessageController(currentClientId, activeChatItem.dataset.id, messageText);
    // Create the sent message element and add it to the chat window.
    const messageDiv = document.createElement('div');
    messageDiv.className = 'message sent';
    //messageDiv.style = 'margin-bottom: 10px; background-color: #007bff; padding: 10px; border-radius: 10px; max-width: 60%; color: white; align-self: flex-end;';
    messageDiv.textContent = messageText;
    chatMessages.appendChild(messageDiv);


    // Scroll to the bottom of the chat window
    chatMessages.scrollTop = chatMessages.scrollHeight;


    // Clear the input field
    messageInput.value = '';


    // Update the history of the active chat.
    if (activeChatItem) {
        // Read the current history or start with an empty array.
        let history = [];
        if (activeChatItem.dataset.history) {
            try {
                history = JSON.parse(activeChatItem.dataset.history);
            } catch (e) {
                console.error("Error parsing chat history:", e);
                history = [];
            }
        }
        // Add the new message to the history.
        history.push({Speaker: "Me", Message: messageText});
        // Save the updated history back as a JSON string.
        activeChatItem.dataset.history = JSON.stringify(history);
    }
}
function sendMessageController(sourceClientId, destClientId, messageText){
    //Chen sending message controller
    if (ws.readyState === WebSocket.OPEN) {
        // Construct the message with the actual values of whichClient and whichServer
        const message = {
            WsSendMessage: {
                source_client_id: sourceClientId.toString(), // Ensure u64 is sent as a string
                dest_client_id: destClientId.toString(),     // Ensure u64 is sent as a string
                message: messageText,
            }
        };
        ws.send(JSON.stringify(message));
        console.log('Sent:', message);
    } else {
        console.error('WebSocket is not open. Unable to send update command.');
    }
}




// UPDATING


function updateChatReceivers(HashListReceivers){
    for (const [serverId, listReceivers] of Object.entries(HashListReceivers)) {
        // Update the phonebooks object.
        phonebooks[serverId] = listReceivers;


        // Stop the reload animation, if present.
        const reloadElem = document.getElementById(`reload-${serverId}`);
        if (reloadElem) {
            reloadElem.style.animation = "";
        }


        // Update the list box options.
        const listBox = document.getElementById(`server-list-box-${serverId}`);
        if (listBox) {
            // Clear existing options.
            listBox.innerHTML = "";


            // Create and append the default "Not Choosed" option.
            const defaultOption = document.createElement('option');
            defaultOption.value = '';
            defaultOption.textContent = 'Not Choosed';
            listBox.appendChild(defaultOption);


            // Populate options with each receiver from listReceivers.
            listReceivers.forEach(receiver => {
                const option = document.createElement('option');
                option.value = receiver;
                option.textContent = receiver;
                listBox.appendChild(option);
            });
        }
    }
}


function updateChats(clientId, ChatHistory) {
    console.log("updating chats, chatHistory: ");
    console.log(ChatHistory)
    // Find the chat item with data-id equal to clientId.
    const chatItem = document.querySelector(`.chat-item[data-id="${clientId}"]`);
    if (!chatItem) {
        // If there is no chat item for this clientId, do nothing.
        return;
    }
    // Get the currently stored history from a custom data attribute (if any).
    let currentHistory = chatItem.dataset.history ? JSON.parse(chatItem.dataset.history) : [];
    // Get the new history from ChatHistory for this client.
    let newHistory = ChatHistory[clientId] || [];


    // Compare histories using JSON.stringify (for simple equality)
    if (JSON.stringify(currentHistory) !== JSON.stringify(newHistory)) {
        // Save the new history in the element's dataset.
        chatItem.dataset.history = JSON.stringify(newHistory);


        // Check if this chat item is active (has class "active").
        if (chatItem.classList.contains('active')) {
            // If active, update the chat window immediately.
            updateChatWindow(newHistory);
        } else {
            // If not active, show the update dot.
            const updateDot = chatItem.querySelector('.update-dot');
            if (updateDot) {
                updateDot.style.display = 'inline-block';
            }
        }
    }
}


function updateChatWindow(history) {
    console.log("ciao")
    const chatMessages = document.getElementById('chat-messages');
    chatMessages.innerHTML = ""; // Clear current messages

    history.forEach(msg => {
        let speaker, message;
        if (Array.isArray(msg)) {
            // If msg is an array, assume the first element is the speaker and the second is the message.
            speaker = msg[0];
            message = msg[1];
        } else {
            // Otherwise, assume it's an object.
            speaker = msg.Speaker;
            message = msg.Message;
        }
        const messageDiv = document.createElement('div');
        messageDiv.className = (speaker === "Me") ? "message sent" : "message received";
        messageDiv.textContent = message;
        chatMessages.appendChild(messageDiv);
    });

    // Scroll to the bottom of the chat messages container.
    chatMessages.scrollTop = chatMessages.scrollHeight;
}


















































/*


   USER VIEW
   -> CONTENT APP: General for now


*/




// SERVER DATA STRUCTURE (Each server has its own files)
const file_lists = [
    /*
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
    }*/
];


const media = [
    /*
    {
        reference : ""
    },
    */
]


// TRACK CURRENT SERVER INDEX
let currentServerIndex = 0;


// FILTER CURRENT and SEARCH CURRENT
let currentFilterType = ""; // An empty string means no filter is applied.
let currentSearchValue = "";




// FUNCTION TO NAVIGATE SERVERS
function navigateServer(direction) {
    // Get an array of server IDs (as numbers) from file_lists,
    // and sort them numerically.
    const serverKeys = Object.keys(file_lists).map(Number).sort((a, b) => a - b);


    // currentServerIndex now is an index into this serverKeys array.
    let newIndex = currentServerIndex + direction;
    if (newIndex < 0) {
        newIndex = serverKeys.length - 1;
    } else if (newIndex >= serverKeys.length) {
        newIndex = 0;
    }
    currentServerIndex = newIndex;
    updateServerDisplay();
}


function get_server_id_from_current_server_index(){
    const serverKeys = Object.keys(file_lists);
    return serverKeys[currentServerIndex];
}




// FUNCTION TO UPDATE UI WHEN SWITCHING SERVERS
function updateServerDisplay() {
    const serverKeys = Object.keys(file_lists).map(Number).sort((a, b) => a - b);
    // Get the actual server ID for the current index.
    const currentServerId = serverKeys[currentServerIndex];
    const currentServer = file_lists[currentServerId];


    // Update Server Name
    document.getElementById("current-server").textContent = currentServer.name;


    // Update File List
    updateFileList(currentServer.files);
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


function openPopup(fileName) {
    const popup = document.getElementById("file-popup");
    const popupTitle = document.getElementById("file-popup-title");
    const popupFileContent = document.getElementById("file-popup-file-content");


    // Get the current server and file content.
    const currentServer = file_lists[get_server_id_from_current_server_index()];
    const fileContent = currentServer.files[fileName];


    popupTitle.textContent = fileName;


    if (fileContent) {


        // Determine the file extension.
        const extension = fileName.split('.').pop().toLowerCase();
        if (extension === "html") {
            // Use a regular expression to find all occurrences of #Media[...] in the content.
            const processedContent = fileContent.replace(/#Media\[(.*?)\]/g, (match, p1) => {
                const found = media.find(item => item.reference === p1);
                console.error("not sure if there can be not the reference inside media when fileContent is present");
                if (found && found.media) {
                    // Use the already loaded media image.
                    return `<img src="${found.media}" id="reference-${p1}" alt="Media loaded" />`;
                } else {
                    // Otherwise, request the media and show a loading image.
                    askMedia(p1);
                    return `<img src="content_objects/reload.png" id="reference-${p1}" alt="Loading..." />`;
                }
            });
            // Insert the processed HTML into the popup.
            popupFileContent.innerHTML = processedContent;
        } else {
            // For other file types, just display the content as text.
            popupFileContent.textContent = fileContent;
        }
    }else{
        popupFileContent.innerHTML = '';
        console.log("qua")
        askFileContent(currentClientId,get_server_id_from_current_server_index(), fileName);
    }


    popup.style.display = "flex";
}

// FUNCTION TO OPEN POP-UP WITH FILE CONTENT
function openPopup(fileName) {
    const popup = document.getElementById("file-popup");
    const popupTitle = document.getElementById("file-popup-title");
    const popupFileContent = document.getElementById("file-popup-file-content");

    // Get the current server and file content.
    const currentServer = file_lists[get_server_id_from_current_server_index()];
    const fileContent = currentServer.files[fileName];

    popupTitle.textContent = fileName;


    if (fileContent) {


        // Determine the file extension.
        const extension = fileName.split('.').pop().toLowerCase();
        if (extension === "html") {
            // Use a regular expression to find all occurrences of #Media[...] in the content.
            const processedContent = fileContent.replace(/#Media\[(.*?)\]/g, (match, p1) => {
                const found = media.find(item => item.reference === p1);
                console.error("not sure if there can be not the reference inside media when fileContent is present");
                if (found && found.media) {
                    // Use the already loaded media image.
                    return `<img src="${found.media}" id="reference-${p1}" alt="Media loaded" />`;
                } else {
                    // Otherwise, request the media and show a loading image.
                    askMedia(p1);
                    return `<img src="content_objects/reload.png" id="reference-${p1}" alt="Loading..." />`;
                }
            });
            // Insert the processed HTML into the popup.
            popupFileContent.innerHTML = processedContent;
        } else {
            // For other file types, just display the content as text.
            popupFileContent.textContent = fileContent;
        }
    }else{
        popupFileContent.innerHTML = '';
        console.log("qua")
        askFileContent(currentClientId,get_server_id_from_current_server_index(), fileName);
    }

    popup.style.display = "flex";
}


// Close the Pop-Up
function closePopup() {
    document.getElementById("file-popup").style.display = "none";
}


// Reload
function reloadFilesServer(whichServer) {
    // Call the empty function
    askFileList(currentClientId, whichServer);


    // Show the loading overlay pop-up
    const loadingPopup = document.getElementById("loading-popup");
    loadingPopup.style.display = "flex";


}


//ASKING


function askFileList(clientId, serverId) {
    if (ws.readyState === WebSocket.OPEN) {
        // Use the correct format for serde deserialization
        const message = {
            WsAskFileList: {
                client_id: clientId.toString(), // Ensure u64 is sent as a string
                server_id: serverId.toString(), // Ensure u64 is sent as a string
            }
        };

        ws.send(JSON.stringify(message));
        console.log("Sent:", JSON.stringify(message)); // Debugging output
    } else {
        console.error("WebSocket is not open. Unable to send request.");
    }
}
function askFileContent(clientId, serverId, fileName) {
    //Chen ask file to controller (file name with .extension given)
    if (ws.readyState === WebSocket.OPEN) {
        // Construct the message with the actual values of whichClient and whichServer
        const message = {
            WsAskFileContent: {
                client_id: clientId.toString(), // Ensure u64 is sent as a string
                server_id: serverId.toString(), // Ensure u64 is sent as a string
                file_ref: fileName,
            }
        };
        ws.send(JSON.stringify(message));
        console.log('Sent:', message);
    } else {
        console.error('WebSocket is not open. Unable to send update command.');
    }
}


function askMedia(clientId, reference_media){
    //Chen ask media to controller
    if (ws.readyState === WebSocket.OPEN) {
        // Construct the message with the actual values of whichClient and whichServer
        const message = {
            WsAskMedia: {
                client_id: clientId.toString(), // Ensure u64 is sent as a string
                media_ref: reference_media,
            }
        };
        ws.send(JSON.stringify(message));
        console.log('Sent:', message);
    } else {
        console.error('WebSocket is not open. Unable to send update command.');
    }
}




























/*
// INITIALIZE UI WITH FIRST SERVER
document.addEventListener("DOMContentLoaded", updateServerDisplay);
*/




// FUNCTION TO UPDATE FILE LIST BASED ON SERVER
function updateFileList(files) {
    // 'files' is expected to be an array of file names.
    const fileListTable = document.querySelector(".file-list");
    fileListTable.innerHTML = ""; // Clear existing content

    // Get the sorted array of server IDs from file_lists.
    const serverKeys = Object.keys(file_lists).map(Number).sort((a, b) => a - b);
    // Get the current server ID from the sorted array.
    const currentServerId = serverKeys[currentServerIndex];

    console.log(`Received files: ${files}`);

    // Update the file list for the current server.
    if (file_lists[currentServerId]) {
        file_lists[currentServerId].files = files;
    } else {
        console.error("No server found for ID", currentServerId);
    }

    // Populate the new file list filtering by the currentFilterType and currentSearchValue.
    files.forEach(fileName => {
        // If a filter is set, check the file extension.
        if (currentFilterType) {
            const extension = fileName.split('.').pop().toLowerCase();
            if (extension !== currentFilterType.toLowerCase()) {
                return;  // Skip files that don't match the filter.
            }
        }

        // Check if the file name contains the search term.
        if (currentSearchValue) {
            if (!fileName.toLowerCase().includes(currentSearchValue.toLowerCase())) {
                return; // Skip if the file name doesn't match.
            }
        }

        const row = document.createElement("tr");
        row.innerHTML = `
           <td>${fileName}</td>
           <td style="text-align:right;">${file_lists[get_server_id_from_current_server_index()].name}</td>
        `;
        // When the row is clicked, open the popup with that file.
        row.addEventListener("click", () => openPopup(fileName));
        fileListTable.appendChild(row);
    });

    // Stop and hide the loading popup.
    const loadingPopup = document.getElementById("loading-popup");
    if (loadingPopup.style.display === "flex") {
        loadingPopup.style.display = "none";
    }
}


function updateFile(file_content) {
    // Get the file name from the popup title.
    const fileName = document.getElementById("file-popup-title").textContent.trim();

    // Get the current server from file_lists.
    const serverKeys = Object.keys(file_lists).map(Number).sort((a, b) => a - b);
    const currentServerId = serverKeys[currentServerIndex];
    const currentServer = file_lists[currentServerId];

    // Update the file content if this file exists.
    if (currentServer && currentServer.files && currentServer.files.hasOwnProperty(fileName)) {
        currentServer.files[fileName] = file_content;
    }

    // Update the popup with the new content.
    const popupFileContent = document.getElementById("file-popup-file-content");
    const extension = fileName.split('.').pop().toLowerCase();
    if (extension === "html") {
        const parts = file_content.split(/(#Media\[[^\]]*\])/g);
        const processedContent = parts.map(part => {
            const mediaMatch = part.match(/^#Media\[(.*?)\]$/);
            if (mediaMatch) {
                const reference = mediaMatch[1];
                const found = media.find(item => item.reference === reference);
                if (found && found.media) {
                    return `<img src="${found.media}" id="reference-${reference}" alt="Media loaded" />`;
                } else {
                    askMedia(reference);
                    return `<img src="content_objects/reload.png" class="loading_image" id="reference-${reference}" alt="Loading..." />`;
                }
            } else {
                return part.trim() ? `<p>${part.trim()}</p>` : "";
            }
        }).join("");
        popupFileContent.innerHTML = processedContent;
    } else {
        popupFileContent.textContent = file_content;
    }
}


function updateMedia(mediaRef) {
    // mediaRef is expected to be an array: [reference, base64ImageString]


    console.log(mediaRef);
    console.log(mediaRef[0]);
    console.log(mediaRef[1]);

    const reference = mediaRef[0];
    const base64Image = mediaRef[1];


    // Find the image element with the corresponding id.
    const imgElem = document.getElementById("reference-" + reference);
    if (imgElem) {
        imgElem.src = base64Image;
    } else {
        console.warn("No element found with id:", "reference-" + reference);
    }
}




















// SEARCH AND FILTERS
function searchGoogleDrive() {
    const searchInputValue = document.querySelector('.search-input').value.trim();


    if (currentSearchValue !== searchInputValue && Object.keys(file_lists[get_server_id_from_current_server_index()].files).length !== 0) {
        currentSearchValue = searchInputValue;
        updateFileList(file_lists[get_server_id_from_current_server_index()].files);
    }
}


function handleSearchKeyPress(event) {
    if (event.key === "Enter") {
        searchGoogleDrive();
    }
}




document.addEventListener("DOMContentLoaded", () => {
    document.querySelectorAll('.filter-option').forEach(btn => {
        btn.addEventListener('click', function() {
            // Set the current filter type from the button's data-type attribute.
            currentFilterType = this.dataset.type;
            // Optionally, you can add visual feedback (e.g., add a 'selected' class).
            document.querySelectorAll('.filter-option').forEach(b => b.classList.remove('selected'));
            this.classList.add('selected');
            // Now update the file list for the current server.
            const currentServer = file_lists[get_server_id_from_current_server_index()];
            if (currentServer && currentServer.files) {
                updateFileList(currentServer.files);
            }
        });
    });
});


function logOut() {
    alert("Logging out...");
    window.location.reload(); // Simulating logout by refreshing
}

