



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
        const chatMessages = document.getElementById('chat-messages');
        if (chatMessages.style.background != 'url("content_objects/whatsapp_bg.jpg")'){
            chatMessages.style.background = 'url("content_objects/whatsapp_bg.jpg")';
            chatMessages.style.backgroundSize = 'cover';
            chatMessages.style.backgroundPosition = 'center';
        }

        document.getElementById("chat-messages").innerHTML = '';
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

        receiverList.innerHTML = '';

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


    createChatItem(selectedReceiver);

    // Optionally, open the new chat immediately
    openChat(selectedReceiver, selectedReceiver);

    // Close the chat pop-up
    closeChatPopup();
}

function createChatItem(clientId){
    // Create a new chat item in the chat list (sidebar)
    const chatList = document.getElementById('chat-list');
    const chatItem = document.createElement('div');
    chatItem.className = 'chat-item';
    chatItem.dataset.id = clientId; // assign the data-id attribute


    // Include both the chat name and the update dot
    chatItem.innerHTML = `<span class="chat-name">${clientId}</span>
                         <span class="update-dot" style="display: none;"></span>`;


    // Set the onclick handler to open the chat and remove the notification dot
    chatItem.onclick = () => {
        openChat(clientId, clientId);
        // Remove the update dot when the chat is selected:
        const dot = chatItem.querySelector('.update-dot');
        if (dot) dot.style.display = 'none';
    };


    chatList.appendChild(chatItem);
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
    } else {
        console.error('WebSocket is not open. Unable to send update command.');
    }
}

document.getElementById('message-input').addEventListener('keydown', (event) => {
    if (event.key === 'Enter') {
        sendMessage();
    }
});

function sendMessage() {
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

            const defaultOption = document.createElement('option');
            defaultOption.value = '';
            defaultOption.textContent = 'Not Choosed';
            listBox.appendChild(defaultOption);

            // Gather existing chats (assuming each chat item’s text is the receiver id)
            const existingChats = Array.from(document.getElementById('chat-list').children)
                .map(item => item.dataset.id);

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
        }
    }
}


function updateChats(ChatHistory) {
    const old_histories = Array.from(document.getElementById('chat-list').children)
        .reduce((acc, item) => {
            acc[item.dataset.id] = item.dataset.history;
            return acc;
        }, {});

    // Find the chat items that changed.
    for (const clientId in ChatHistory){
        const newHistory = ChatHistory[clientId];
        if (!old_histories[clientId]){
            createChatItem(clientId)
            const chatItem = document.querySelector(`.chat-item[data-id="${clientId}"]`);
            const updateDot = chatItem.querySelector('.update-dot');
            if (updateDot) {
                updateDot.style.display = 'inline-block';
            }
        }
        if (JSON.stringify(newHistory) !== old_histories[clientId]){
            const chatItem = document.querySelector(`.chat-item[data-id="${clientId}"]`);
            chatItem.dataset.history = JSON.stringify(newHistory);

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
}


function updateChatWindow(history) {
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
    const serverKeys = Object.keys(file_lists);

    if (direction>0){
        if ((currentServerIndex+1) === serverKeys.length){
            currentServerIndex = 0;
        }else{
            currentServerIndex = currentServerIndex + 1;
        }
    }else{

        if ((currentServerIndex-1) < 0){
            currentServerIndex = serverKeys.length -1;
        }else{
            currentServerIndex = currentServerIndex -1;
        }
    }
    updateServerDisplay();
}


function get_server_id_from_current_server_index(){
    const serverKeys = Object.keys(file_lists);

    return serverKeys[currentServerIndex];
}




// FUNCTION TO UPDATE UI WHEN SWITCHING SERVERS
function updateServerDisplay() {
    const currentServerId = get_server_id_from_current_server_index()
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

// FUNCTION TO OPEN POP-UP WITH FILE CONTENT
function  openPopup(fileName) {
    const popup = document.getElementById("file-popup");
    const popupTitle = document.getElementById("file-popup-title");
    const popupFileContent = document.getElementById("file-popup-file-content");

    // Get the current server and file content.
    //console.log(file_lists)
    //console.log(get_server_id_from_current_server_index())
    //console.log(file_lists[get_server_id_from_current_server_index()])

    const currentServer = file_lists[get_server_id_from_current_server_index()];
    //console.log(currentServer.files)
    //console.log(fileName)
    const fileContent = currentServer.files[fileName];

    popupTitle.textContent = fileName;
    popupFileContent.innerHTML = '';

    //console.log(fileContent)
    if (fileContent) {
        // Determine the file extension.
        const extension = fileName.split('.').pop().toLowerCase();
        if (extension === "html") {
            // Use a regular expression to find all occurrences of #Media[...] in the content.
            const processedContent = fileContent.replace(/#Media\[(.*?)\]/g, (match, p1) => {
                const found = media.find(item => item.reference === p1);

                // Use the already loaded media image.
                return `<img src="${found.media}" id="reference-${p1}" alt="Media loaded" />`;
            });
            // Insert the processed HTML into the popup.
            popupFileContent.innerHTML = processedContent;
        } else if (extension === "mp3") {
            // For audio files, embed an HTML5 audio element.
            popupFileContent.innerHTML = `
              <audio controls style="width:100%;">
                <source src="${fileContent}" type="audio/mpeg">
                Your browser does not support the audio element.
              </audio>
            `;
        } else if (extension === "jpg" || extension === "jpeg" || extension === "png" || extension === "gif") {
            // For images, use an img element.
            popupFileContent.innerHTML = `<img src="${fileContent}" alt="${fileName}" style="max-width:100%;" />`;
        } else {
            // For all other types, simply display the content as text.
            popupFileContent.textContent = fileContent;
        }
    } else {
        popupFileContent.innerHTML = '';
        askFileContent(currentClientId, get_server_id_from_current_server_index(), fileName);
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

    // Get the current server ID from the sorted array.
    const currentServerId = get_server_id_from_current_server_index();

    // Update the file list for the current server.
    if (file_lists[currentServerId]) {
        // Create an object where each key is a file name
        const filesObj = {};
        files.forEach((file) => {
            filesObj[file] = ""; // You can store additional info instead of file if needed.
        });
        file_lists[currentServerId].files = filesObj;
    } else {
        console.error("No server found for ID", currentServerId);
    }

    if (Object.keys(files).length !== 0){
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
    }

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
    const currentServerId = get_server_id_from_current_server_index();
    const currentServer = file_lists[currentServerId];

    console.log(currentServer.files)
    console.log(fileName)
    console.log(currentServer.files[fileName])
    console.log(file_content)
    // Update the file content if this file exists.
    currentServer.files[fileName] = file_content;

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
                    // Media is already loaded.
                    return `<img src="${found.media}" id="reference-${reference}" alt="Media loaded" style="width:400px; height:auto" />`;

                } else if (!requestedMedia.has(reference)) {
                    // Request the media only if it hasn't been requested yet.
                    requestedMedia.add(reference); // Mark as requested
                    askMedia(currentClientId, reference); // Request the media
                    return `<img src="content_objects/reload.png" class="loading_image" id="reference-${reference}" alt="Loading..." />`;
                } else {
                    // Media has already been requested but is not yet loaded.
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
    console.log(mediaRef);
    const fullPath = window.location.pathname;
    // Remove the filename (assumes a filename is present)
    const basePath = fullPath.substring(0, fullPath.lastIndexOf('/'));
    // Combine with the protocol
    const absolutePath = window.location.protocol + basePath;

    for (const key in mediaRef){
        const reference = key;
        const base64Image = mediaRef[reference];
        // Add the media to the media array.
        const existingMedia = media.find(item => item.reference === reference);
        if (!existingMedia) {
            media.push({ reference, media: base64Image });
        } else {
            existingMedia.media = absolutePath + base64Image;
        }
        // Find the image element with the corresponding id.
        const imgElem = document.getElementById("reference-" + reference);
        if (imgElem) {
            //console.log(base64Image);
            imgElem.classList.remove("loading", "rotate");  // Remove any rotation classes
            imgElem.style.animation = "none";  // Stop rotation
            imgElem.style.transform = "none";  // Reset any transforms
            imgElem.style.width = "400px";
            imgElem.style.height = "auto";
            imgElem.src = absolutePath + base64Image;
        } else {
            //console.warn("No element found with id:", "reference-" + reference);
        }
    }
}
