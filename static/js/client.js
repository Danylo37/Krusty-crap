

/*

    USER VIEW
    -> WHATSAPP: SEND MESSAGE, CREATING AND SWITCHING BETWEEN CHATS

 */

function openChat(chatName) {
    // Update the chat header with the selected chat name
    document.getElementById('chat-header').textContent = chatName;

    // Update the chat messages area with a new background and placeholder messages
    const chatMessages = document.getElementById('chat-messages');
    chatMessages.innerHTML = `
                <div class="message received">Welcome to ${chatName}!</div>
                <div class="message sent">Hi there!</div>
            `;

    chatMessages.style.background = 'url("whatsapp_bg.jpg")';
    chatMessages.style.backgroundSize = 'cover';
    chatMessages.style.backgroundPosition = 'center';

    // Highlight the active chat in the sidebar
    const chatItems = document.querySelectorAll('.chat-item');
    chatItems.forEach((item) => item.classList.remove('active')); // Remove active class from all
    const selectedChat = [...chatItems].find((item) => item.textContent === chatName);
    if (selectedChat) selectedChat.classList.add('active');
}



// Phonebooks STORED
const phonebooks = {
    "Server1": ["Mom", "Dad", "Sibling"],
    "Server2": ["Alice", "Bob", "Charlie"],
    "Server3": ["Manager", "Colleague A", "Colleague B"],
};


function createNewChat() {
    const chatPopup = document.getElementById('chat-popup');
    const receiverList = document.getElementById('receiver-list');
    receiverList.innerHTML = ''; // Clear existing content

    const listBoxes = [];

    // Create list boxes dynamically for each server
    Object.keys(phonebooks).forEach((serverName) => {
        // Server container
        const serverContainer = document.createElement('div');
        serverContainer.style = "margin-bottom: 15px; text-align: left;";

        // Server label
        const serverLabel = document.createElement('label');
        serverLabel.textContent = `${serverName}: `;
        serverLabel.style = "font-weight: bold;";

        // Server list box
        const listBox = document.createElement('select');
        listBox.className = 'server-list-box';
        listBox.style = "margin-left: 10px; padding: 5px;";

        // Default "Not Choosed" option
        const defaultOption = document.createElement('option');
        defaultOption.value = '';
        defaultOption.textContent = 'Not Choosed';
        listBox.appendChild(defaultOption);

        // Populate options with receivers
        phonebooks[serverName].forEach((receiver) => {
            const option = document.createElement('option');
            option.value = receiver;
            option.textContent = receiver;
            listBox.appendChild(option);
        });

        // Add change event to handle exclusivity
        listBox.addEventListener('change', () => {
            listBoxes.forEach((box) => {
                if (box !== listBox) {
                    box.value = ''; // Reset others
                    box.disabled = listBox.value !== ''; // Disable if another is chosen
                }
            });
        });

        listBoxes.push(listBox);

        // Append label and list box to the server container
        serverContainer.appendChild(serverLabel);
        serverContainer.appendChild(listBox);

        // Append server container to the receiver list
        receiverList.appendChild(serverContainer);
    });

    chatPopup.style.display = 'flex'; // Show the popup
}

function confirmSelection() {
    const listBoxes = document.querySelectorAll('.server-list-box');
    let selectedReceiver = '';

    // Get the selected receiver
    listBoxes.forEach((box) => {
        if (box.value) {
            selectedReceiver = box.value;
        }
    });

    if (!selectedReceiver) {
        alert('Please select a receiver.');
        return;
    }

    console.log(`Selected Receiver: ${selectedReceiver}`);

    // Show blank page
    const clientView = document.getElementById('client-view');
    clientView.innerHTML = `
                <div style="height: 100vh; display: flex; justify-content: center; align-items: center;">
                    <h1>Loading...</h1>
                </div>
            `;
    clientView.style.display = 'flex';

    // Update chats before showing the WhatsApp UI
    updateChats();

    // Simulate loading (You can replace this with actual logic to show the WhatsApp UI)
    setTimeout(() => {
        clientView.innerHTML = ''; // Clear the blank page
        document.getElementById('communication-ui').style.display = 'flex'; // Show WhatsApp UI
    }, 1000);
}

function closeChatPopup() {
    const chatPopup = document.getElementById('chat-popup');
    chatPopup.style.display = 'none'; // Hide the popup
}

function sendMessage() {
    const messageInput = document.getElementById('message-input');
    const chatMessages = document.getElementById('chat-messages');
    const messageText = messageInput.value.trim();

    if (messageText) {
        // Add the sent message to the chat window
        const messageDiv = document.createElement('div');
        messageDiv.className = 'message sent';
        messageDiv.style = 'margin-bottom: 10px; background-color: #007bff; padding: 10px; border-radius: 10px; max-width: 60%; color: white; align-self: flex-end;';
        messageDiv.textContent = messageText;

        chatMessages.appendChild(messageDiv);

        // Scroll to the bottom of the chat window
        chatMessages.scrollTop = chatMessages.scrollHeight;

        // Clear the input field
        messageInput.value = '';
    }
}

