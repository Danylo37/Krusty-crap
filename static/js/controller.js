/*

    CONTROLLER VIEW
    -> button for switching between modes

 */


document.querySelector(".previewContainer").addEventListener("click", function() {
    const network_container = document.querySelector('#network-container');
    const monitoring_container = document.querySelector('#monitoring-container');
    const previewButton = document.querySelector('.previewButton');

    if (network_container.style.display === 'none') {
        previewButton.classList.remove('show-front');
        previewButton.classList.add('show-back');
        monitoring_container.style.display = 'none';
        network_container.style.display = 'block';
    } else {
        previewButton.classList.remove('show-back');
        previewButton.classList.add('show-front');
        network_container.style.display = 'none';
        monitoring_container.style.display = 'block';
    }
});











/* GENERAL FUNCTIONS FOR CONTROLLER */
const fullPath = window.location.pathname;
// Remove the filename (assumes a filename is present)
const basePath = fullPath.substring(0, fullPath.lastIndexOf('/'));
// Combine with the protocol
const absolutePath = window.location.protocol + basePath;





let droneCrashed = "false";
function crashDrone(drone) {

    droneCrashed = "false";
    const canvas = document.getElementById("network-canvas");

    const explosion = createExplosionGif(drone.x, drone.y, canvas);
    const airplaneElements= createAirplanes(drone.x, drone.y, canvas);

    sendCrashController(drone.id);


    const intervalId = setInterval(() => {
        // Check if the global variable "droneCrashed" is set to "droneCrashed".
        if (droneCrashed === "droneCrashed") {
            // Remove the explosion GIF.
            if (canvas.contains(explosion)) {
                canvas.removeChild(explosion);
            }
            // Remove all airplane elements.
            airplaneElements.forEach(plane => {
                if (canvas.contains(plane)) {
                    canvas.removeChild(plane);
                }
            });
            // Remove the drone from your topology and from the UI.
            removeDroneFromTopology(drone);
            removeDroneFromSection();

            // Stop the interval.
            clearInterval(intervalId);
        }else if (droneCrashed === "droneNotCrashed"){

            // Remove the explosion GIF.
            if (canvas.contains(explosion)) {
                canvas.removeChild(explosion);
            }
            // Remove all airplane elements.
            airplaneElements.forEach(plane => {
                if (canvas.contains(plane)) {
                    canvas.removeChild(plane);
                }
            });

            alert("Drone was too powerful sorry, can't be crashed");

            const nononoImg = document.getElementById('nonono');

            nononoImg.classList.remove('animate1');
            nononoImg.classList.remove('animate2');

            nononoImg.classList.add('animate1');
            setTimeout(() => {
                nononoImg.classList.add('animate2');
            }, 3000);

            // Stop the interval.
            clearInterval(intervalId);
        }
    }, 3000);
}

function createExplosionGif(droneX, droneY, canvas){

    // Create an SVG image element for the explosion GIF.
    const explosion = document.createElementNS("http://www.w3.org/2000/svg", "image");
    explosion.setAttributeNS(null, "href", "content_objects/explosion.gif");
    // Position the explosion so that it covers the drone (adjust the offset as needed).
    explosion.setAttribute("x", droneX - 30);
    explosion.setAttribute("y", droneY - 30);
    explosion.setAttribute("width", "60");
    explosion.setAttribute("height", "60");
    explosion.classList.add("explosion");
    canvas.appendChild(explosion);

    return explosion;
}

function createAirplanes(droneX, droneY, canvas){
    // Create 3 airplane elements that will orbit the drone.
    const airplaneElements = [];
    for (let i = 0; i < 3; i++) {
        const airplane = document.createElementNS("http://www.w3.org/2000/svg", "image");
        airplane.setAttributeNS(null, "href", "content_objects/airplane.png");
        airplane.setAttribute("width", "60");
        airplane.setAttribute("height", "60");
        airplane.classList.add("airplane");
        // Calculate an initial position offset for each airplane.
        // For example, use angles 0, 120, and 240 degrees.
        const angle = (2 * Math.PI * i) / 3;
        const offsetX = 40 * Math.cos(angle) - 40;
        const offsetY = 20 * Math.sin(angle) - 20;
        airplane.setAttribute("x", droneX + offsetX);
        airplane.setAttribute("y", droneY + offsetY);
        // Add a CSS class to animate the airplane along an ellipse.
        airplane.classList.add("ellipse-animation");
        canvas.appendChild(airplane);
        airplaneElements.push(airplane);
    }
    return airplaneElements;
}

function sendCrashController(droneId){
    if (ws.readyState === WebSocket.OPEN) {
        const message = {
            WsCrashDrone: {
                drone_id: droneId.toString(),
            }
        };
        ws.send(JSON.stringify(message));
        console.log('Sent:', message);
    } else {
        console.error('WebSocket is not open. Unable to send update command.');
    }
}



/* UPDATES */

function updateCrashedDrone(isCrashed, parsedData){
    if (isCrashed){
        droneCrashed = "droneCrashed";
    }else{
        droneCrashed = "droneNotCrashed"
    }
}

function updatePdrDrone(parsedData) {
    // Expect parsedData to include node_id and pdr
    const nodeId = parsedData.node_id;
    const newPdr = parsedData.pdr;

    // 1. Update the drawn_nodes array.
    const droneNode = drawn_nodes.find(node => node.id == nodeId);
    if (droneNode) {
        droneNode.pdr = newPdr;
    }

    // 2. Update the monitoring panel's dataset if it exists.
    const panel = document.querySelector(`.panel[data-node-id="${nodeId}"]`);
    if (panel) {
        panel.dataset.pdr = newPdr;
    }

    // 3. If the drone details side-tab is visible and showing this drone, update its displayed value.
    const droneDetailsTab = document.getElementById('drone-details');
    if (droneDetailsTab && droneDetailsTab.style.display !== 'none') {
        const currentDroneId = document.getElementById('drone-id').textContent;
        if (currentDroneId == nodeId) {
            document.getElementById('drone-pdr').textContent = newPdr;
        }
    }
}

function crashDroneReview(){
    droneCrashed = "droneNotCrashed";
}











// Toggle the legend container open/closed
function toggleLegend() {
    const legendContainer = document.getElementById('legend-container');

    topologyImagesToggled = !topologyImagesToggled
    updateLegend()
    topologyImagesToggled = !topologyImagesToggled

    legendContainer.classList.toggle('expanded');

    // Optionally, rotate the down arrow for visual feedback:
    const downArrow = document.getElementById('down-arrow-btn');
    if (legendContainer.classList.contains('expanded')) {
        downArrow.style.transform = 'rotate(180deg)';
    } else {
        downArrow.style.transform = 'rotate(0deg)';
    }
}

// Update the legend images based on the current topology appearance mode
function updateLegend() {
    const legendItems = document.querySelectorAll('#legend-container .legend-item');
    legendItems.forEach(item => {
        const label = item.querySelector('span').textContent.trim();
        // Get the first child element which is either an <img> or a div (for the circle)
        let iconElem = item.querySelector('.legend-icon, .legend-circle');

        if (topologyImagesToggled) {
            // When topologyImagesToggled is TRUE, show a circle with a fill color.
            let fillColor = '';
            switch(label) {
                case "Drone":
                    fillColor = "#f39c12";
                    break;
                case "Communication Server":
                    fillColor = "#e74c3c";
                    break;
                case "Chat Client":
                    fillColor = "#9b59b6";
                    break;
                case "Web Browser":
                    fillColor = "#3498db";
                    break;
                case "Text Server":
                    fillColor = "#2ecc71";
                    break;
                case "Media Server":
                    fillColor = "#e67e22";
                    break;
                default:
                    fillColor = "#007bff";
            }
            // If the current element is not already a circle div, replace it.
            if (!iconElem || iconElem.tagName.toLowerCase() !== 'div') {
                // If an image exists, remove it and create a div.
                const newDiv = document.createElement('div');
                newDiv.classList.add('legend-circle');
                // Replace the old element (if any) with the new div.
                if (iconElem) {
                    iconElem.parentNode.replaceChild(newDiv, iconElem);
                } else {
                    // If no icon element exists, just prepend it.
                    item.insertBefore(newDiv, item.firstChild);
                }
                iconElem = newDiv;
            }
            // Apply the circle styling inline (or you can use CSS)
            iconElem.style.backgroundColor = fillColor;
            iconElem.style.width = "30px";
            iconElem.style.height = "30px";
            iconElem.style.borderRadius = "50%";
            iconElem.style.display = "inline-block";
            iconElem.style.marginRight = "10px";
        } else {
            // When topologyImagesToggled is FALSE, show images.
            let src = "";
            switch(label) {
                case "Drone":
                    src = "content_objects/drone_top.jpeg";
                    break;
                case "Communication Server":
                    src = "content_objects/comm_serv_top.jpeg";
                    break;
                case "Chat Client":
                    src = "content_objects/chat_client_top.jpeg";
                    break;
                case "Web Browser":
                    src = "content_objects/web_browser_top.jpeg";
                    break;
                case "Text Server":
                    src = "content_objects/text_serv_top.jpeg";
                    break;
                case "Media Server":
                    src = "content_objects/media_serv_top.jpeg";
                    break;
                default:
                    src = "content_objects/default_top.jpeg";
            }
            // If the current element is not an <img>, replace it.
            if (!iconElem || iconElem.tagName.toLowerCase() !== 'img') {
                const newImg = document.createElement('img');
                newImg.classList.add('legend-icon');
                newImg.src = src;
                newImg.setAttribute("width", "50");
                newImg.setAttribute("height", "50");
                newImg.setAttribute("border-radius", "30");
                // Replace the current element (if exists) or insert it.
                if (iconElem) {
                    iconElem.parentNode.replaceChild(newImg, iconElem);
                } else {
                    item.insertBefore(newImg, item.firstChild);
                }
                iconElem = newImg;
            } else {
                // If it is already an image, simply update its src.
                iconElem.src = src;
            }
        }
    });
}
















/*

    CONTROLLER VIEW
    -> Graph modality

 */


//"Tracking" variables
let translateX = 0; // Horizontal panning offset
let translateY = 0; // Vertical panning offset
let scale = 1; // Initial zoom level
let isPanning = false;
let startX, startY;
let currentTranslateX = 0; // Smoothed horizontal panning offset
let currentTranslateY = 0; // Smoothed vertical panning offset
let panAnimationFrame; // For requestAnimationFrame
const dampingFactor = 0.15; // Damping factor for smoother movement

const container = document.getElementById("network-container");
const canvas = document.getElementById("network-canvas");


// Global arrays for nodes and connections
const drawn_nodes = [];
const connections = [];

/*
//TESTING VARIABLES
const drawn_nodes = [
    { id: "C1", type: "client", x: 100, y: 400 },
    { id: "S1", type: "server", x: 900, y: 400 },
    { id: "D1", type: "drone", x: 500, y: 300 },
    { id: "D2", type: "drone", x: 400, y: 200 },
    { id: "D3", type: "drone", x: 600, y: 200 },
];

const connections = [
    { from: "C1", to: "D1" },
    { from: "S1", to: "D1" },
    { from: "D1", to: "D2" },
    { from: "D1", to: "D3" },
];
*/




// Zoom in/out
const zoomSpeed = 0.08; // Adjust for smoother zooming

document.getElementById("network-container").addEventListener("wheel", (e) => {
    e.preventDefault(); // Prevent default scrolling behavior

    // Get mouse position relative to the container
    const containerRect = container.getBoundingClientRect();
    const canvasRect = canvas.getBoundingClientRect();

    const mouseX = e.clientX - containerRect.left;
    const mouseY = e.clientY - containerRect.top;

    console.log(`Mouse X: ${mouseX}, Mouse Y: ${mouseY}`);

    // Determine zoom direction
    const delta = e.deltaY < 0 ? zoomSpeed : -zoomSpeed;
    const newScale = Math.min(Math.max(scale + delta, 0.5), 2); // Clamp scale between 0.5 and 2

    // Calculate scale factor
    const shiftX = mouseX * delta; // Adjust X based on mouse position and delta
    const shiftY = mouseY * delta; // Adjust Y based on mouse position and delta
    translateX -= shiftX;
    translateY -= shiftY;
    const mousePercentage = (mouseX / ((1/2)*containerRect.right));
    const supposedShift = delta*canvasRect.width;

    //translateX = translateX + (-1)*(mousePercentage * supposedShift);
    console.log(`${scale-newScale}, \ncontainerRect.bottom: ${containerRect.bottom}, canvasRect.bottom: ${canvasRect.bottom}   \ncontainerRect.right: ${containerRect.right}, canvasRect.right: ${canvasRect.right} `);

    console.log(`ratio mouse: ${mouseX / ((1/2)*containerRect.right)}`);
    console.log(` supposed shift: ${canvasRect.width * delta}`);
    let oldVal = canvasRect.width*scale;
    console.log(`Shift done: ${(((mouseX / ((1/2)*containerRect.right)) * (canvasRect.width * -delta)))}`)

    //translateX = translateX + () * ((canvasRect.width*scale) * -delta));
    //const transformY = (1 - (mouseY / containerRect.bottom)) * (canvaYLength * -delta);


    //translateX = translateX + (mouseX/((1/2)*containerRect.right))*((canvasRect.right*(-delta)))//(mouseX/containerRect.bottom)*(canvasRect.bottom*(-delta));


    //translateY = 0//translateY + mouseY*((canvasRect.bottom*(-delta)/containerRect.bottom));
    /*const scaleFactor = newScale / scale;
    const mouseCanvasYBefore = (mouseY - translateY) / scale; // Mouse position in canvas space
    translateY = mouseY - mouseCanvasYBefore * newScale;
    // Adjust translateX and translateY to center zoom at the mouse pointer
    translateX = mouseX - scaleFactor * (mouseX - translateX);
    //translateY = mouseY - scaleFactor * (mouseY - translateY);
    */

    // Apply the new scale and translation
    scale = newScale;

    canvas.style.transform = `translate(${translateX}px, ${translateY}px) scale(${scale})`;
    console.log(`Difference after: ${(canvasRect.width*scale) -oldVal}`);
    console.log({

        mouseX,
        mouseY,
        translateX,
        translateY,
        scale
    });
});


// Start panning
container.addEventListener("mousedown", (e) => {
    isPanning = true;
    startX = e.clientX;
    startY = e.clientY;

    // Cancel any existing animation frame
    if (panAnimationFrame) cancelAnimationFrame(panAnimationFrame);
});

// Perform panning
document.addEventListener("mousemove", (e) => {
    if (!isPanning) return;

    if (e.target.closest("#drone-details")) return;

    // Calculate the difference between the current and starting mouse positions
    const dx = (e.clientX - startX) / scale; // Adjust for current zoom level
    const dy = (e.clientY - startY) / scale;

    // Update target translation values
    translateX += dx;
    translateY += dy;

    // Update the starting position for the next frame
    startX = e.clientX;
    startY = e.clientY;

    // Start the smooth transition using requestAnimationFrame
    smoothPan();

});

// Smooth panning using requestAnimationFrame
function smoothPan() {
    // Update current translation values towards the target
    currentTranslateX += (translateX - currentTranslateX) * dampingFactor;
    currentTranslateY += (translateY - currentTranslateY) * dampingFactor;

    // Apply the transformation
    canvas.style.transform = `translate(${currentTranslateX}px, ${currentTranslateY}px) scale(${scale})`;

    // Continue animation if there's a significant difference
    if (Math.abs(translateX - currentTranslateX) > 0.1 || Math.abs(translateY - currentTranslateY) > 0.1) {
        panAnimationFrame = requestAnimationFrame(smoothPan);
    } else {
        // Snap to the target values when close enough
        currentTranslateX = translateX;
        currentTranslateY = translateY;
    }
}

// Stop panning
document.addEventListener("mouseup", () => {
    isPanning = false;
});


function showDroneDetails(droneData) {

    console.log(droneData)
    // droneData is an object that might look like:
    // { id: 17, type: 'Drone', x: 500, y: 300, ... }
    document.getElementById('drone-id').textContent = droneData.id;
    document.getElementById('drone-type').textContent = droneData.type;
    document.getElementById('drone-coordinates').textContent = `(${droneData.x}, ${droneData.y})`;
    // If you have more fields, update them here.

    // Optionally, update the title or add additional content
    document.getElementById('drone-title').textContent = `Drone ${droneData.id} Details`;
    document.getElementById("crash-btn").onclick = () => crashDrone(droneData);

    // Show the side tab
    document.getElementById('drone-details').style.display = 'block';
}

function hideDroneDetails() {
    document.getElementById('drone-details').style.display = 'none';
}


function removeDroneFromTopology(drone) {
    // Remove the drone from the drawn_nodes array.
    const index = drawn_nodes.findIndex(n => n.id === drone.id);
    if (index !== -1) {
        drawn_nodes.splice(index, 1);
    }

    // Get the SVG canvas.
    const canvas = document.getElementById("network-canvas");

    // Remove the corresponding circle.
    const circles = canvas.getElementsByTagName("circle");
    for (let i = circles.length - 1; i >= 0; i--) {
        // Here, we check by position; if you can, add a data attribute (e.g., data-id) to the circle when creating it.
        if (circles[i].getAttribute("cx") == drone.x && circles[i].getAttribute("cy") == drone.y) {
            canvas.removeChild(circles[i]);
        }
    }

    const imageElem = document.querySelector(`#network-canvas [data-node-id="${drone.id}"]`);
    if (imageElem) {
        imageElem.remove();
    }

    // Remove all lines (connections) associated with the drone.
    const lines = canvas.getElementsByClassName("connection");
    // Convert HTMLCollection to an array (since we'll be removing elements)
    const linesArray = Array.from(lines);
    linesArray.forEach(line => {
        if (line.dataset.from === drone.id || line.dataset.to === drone.id) {
            canvas.removeChild(line);
        }
    });

    // Also remove connections from the array.
    for (let i = connections.length - 1; i >= 0; i--) {
        if (connections[i].from === drone.id || connections[i].to === drone.id) {
            connections.splice(i, 1);
        }
    }
}


















/*

    CONTROLLER VIEW
    -> DROPDOWN MENU

*/


// Function to toggle the dropdown menu visibility
function toggleDropdown() {
    const dropdownMenu = document.getElementById('dropdown-menu');

    if (dropdownMenu.classList.contains('active')) {
        // Start slide-out animation
        dropdownMenu.classList.remove('active');
        dropdownMenu.classList.add('closing');
    } else {
        // Start slide-in animation
        dropdownMenu.style.display = 'block'; // Ensure the content is displayed
        dropdownMenu.classList.add('active');
        dropdownMenu.classList.remove('closing')
    }
}

function toggleTopologyAppearanceButton(){


    toggleTopologyAppearance()
    updateLegend();

    // Toggle the state.
    topologyImagesToggled = !topologyImagesToggled;

}

// Global flag to know which appearance is active.
let topologyImagesToggled = false;

// This function toggles the appearance of nodes on the SVG canvas.
function toggleTopologyAppearance() {
    const canvas = document.getElementById("network-canvas");
    // Iterate over all drawn nodes (which you stored in drawn_nodes).
    drawn_nodes.forEach(node => {

        let currentElem = canvas.querySelector(`.node[data-node-id="${node.id}"]`);
        if (!currentElem) {
            return; // Nothing found? Skip.
        }

        if (!topologyImagesToggled) {
            document.getElementById("img_appearance").src = "content_objects/drone_top.jpeg"
            // --- Switch from circle to image ---
            const imgElem = document.createElementNS("http://www.w3.org/2000/svg", "image");
            imgElem.setAttribute("width", "45");
            imgElem.setAttribute("height", "45");
            imgElem.style.borderRadius = "30px";

            // Choose an image source based on the node type.
            let src = "";
            switch (node.type) {
                case "Drone":
                    src = "content_objects/drone_top.jpeg";
                    break;
                case "CommunicationServer":
                    src = "content_objects/comm_serv_top.jpeg";
                    break;
                case "ChatClient":
                    src = "content_objects/chat_client_top.jpeg";
                    break;
                case "WebBrowser":
                    src = "content_objects/web_browser_top.jpeg";
                    break;
                case "TextServer":
                    src = "content_objects/text_serv_top.jpeg";
                    break;
                case "MediaServer":
                    src = "content_objects/media_serv_top.jpeg";
                    break;
                default:
                    // Fallback image if needed.
                    src = "content_objects/default_top.png";
            }

            imgElem.setAttribute("href", src);

            // Position the image so that its center is at (node.x, node.y).
            imgElem.setAttribute("x", node.x - 15);
            imgElem.setAttribute("y", node.y - 15);

            // Attaching data for easy handling
            imgElem.dataset.nodeId = node.id;
            imgElem.classList.add("node", node.type);

            // Set up a click handler similar to what you had for circles.
            if (node.type === "Drone") {
                imgElem.addEventListener("click", () => showDroneDetails(node));
            } else {
                imgElem.addEventListener("click", () => alert(`Node: ${node.id}, Type: ${node.type}`));
            }

            // Replace the current circle with the image element.
            canvas.replaceChild(imgElem, currentElem);
        } else {
            document.getElementById("img_appearance").src = "content_objects/preview_circle_form.png"
            // --- Revert from image to circle ---
            let imgElem = currentElem;

            // Create a new circle element.
            const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
            circle.setAttribute("cx", node.x);
            circle.setAttribute("cy", node.y);
            circle.setAttribute("r", 15);
            circle.classList.add("node", node.type);
            circle.dataset.nodeId = node.id;

            if (node.type === "Drone") {
                circle.addEventListener("click", () => showDroneDetails(node));
            } else {
                circle.addEventListener("click", () => alert(`Node: ${node.id}, Type: ${node.type}`));
            }

            // Replace the image with the circle.
            canvas.replaceChild(circle, imgElem);
        }
    });
}

// Global variable to track the current layout mode.
let currentLayout = "circle"; // "circle" is the default layout

function toggleTopologyLayout() {
    // Check the current layout mode and switch:
    if (currentLayout === "circle") {
        currentLayout = "grid";
        document.getElementById("img_layout").src = "content_objects/grid_layout.png"
        createTopologyGrid(globalTopologyData);
    } else {
        currentLayout = "circle";
        document.getElementById("img_layout").src = "content_objects/decagram_layout.png"
        createTopology(globalTopologyData);
    }
}























/*

    CONTROLLER VIEW
    -> MONITORING (only switching sections for now)

 */

function showSection(sectionId) {
    const newSection = document.getElementById(sectionId);
    const oldSection = document.querySelector('.section.active');

    // If there's no active section or the new section is already active, do nothing.
    if (!oldSection || oldSection === newSection) {
        newSection.style.display = 'block';
        newSection.classList.add('active');
        return;
    }

    // Start closing animation on the old section.
    oldSection.classList.add('closing');

    // Wait until the closing animation is finished.
    oldSection.addEventListener('animationend', function handleAnimationEnd() {
        // Hide the old section and remove its classes.
        oldSection.style.display = 'none';
        oldSection.classList.remove('active', 'closing');

        // Show and activate the new section.
        newSection.style.display = 'block';
        newSection.classList.add('active');
    }, { once: true });
}

function updatePanelContent(panel, fields) {
    let fieldsContainer = panel.querySelector(".fields-container");

    // If it doesn't exist, create one and append it to the panel
    if (!fieldsContainer) {
        fieldsContainer = document.createElement("div");
        fieldsContainer.classList.add("fields-container");
        panel.appendChild(fieldsContainer);
    }
    fieldsContainer.innerHTML = "";


    Object.entries(fields).forEach(([key, value]) => {
        const field = document.createElement("p");
        field.style.overflowWrap = "break-word";  // Ensure long text wraps
        field.textContent = `${key}: ${JSON.stringify(value)}`;
        fieldsContainer.appendChild(field);
    });
}

function removeDroneFromSection(){
    //console.error("To fill drone from section removing")
}


/* FILTERING MONITORING SECTION */

// Global variable to keep track of sort order for node_id filtering.
let nodeIdSortOrder = 'asc'; // 'asc' (ascending) by default

// Helper function to get the currently visible section container.
function getCurrentSectionContainer() {
    const containerIds = ['clients-container', 'servers-container', 'drones-container'];
    for (let id of containerIds) {
        const el = document.getElementById(id);
        // Assuming only one of these containers is visible (display not 'none')
        if (el && el.style.display !== 'none') {
            return el;
        }
    }
    return null;
}

// When the DOM content is loaded, set up the first filter button.
document.addEventListener('DOMContentLoaded', () => {
    // Assume the first filter button is used for node_id ordering.
    const filterButtons = document.querySelectorAll('#filters .filter-button');
    if (filterButtons.length > 0) {
        const orderButton = filterButtons[0];
        // Set the initial sort order state on the button.
        orderButton.dataset.sortOrder = 'asc';

        orderButton.addEventListener('click', function () {
            if (this.dataset.sortOrder === 'asc') {
                // If current state is ascending, order descending.
                orderPanelsByNodeId('desc');
                this.dataset.sortOrder = 'desc';
                // Rotate the image inside the button if present.
                const img = this.querySelector('img');
                if (img) {
                    img.style.transform = 'rotate(180deg)';
                }
            } else {
                // Otherwise, order ascending.
                orderPanelsByNodeId('asc');
                this.dataset.sortOrder = 'asc';
                const img = this.querySelector('img');
                if (img) {
                    img.style.transform = 'rotate(0deg)';
                }
            }
        });
    }
});


function orderPanelsByNodeId(order = 'asc') {
    const container = getCurrentSectionContainer();
    if (!container) return;

    // Get all panels in the container (panels are assumed to have class 'panel').
    const panels = Array.from(container.getElementsByClassName('panel'));

    // Sort panels by their dataset.nodeId numerically.
    panels.sort((a, b) => {
        const aId = parseInt(a.dataset.nodeId, 10);
        const bId = parseInt(b.dataset.nodeId, 10);
        return order === 'asc' ? aId - bId : bId - aId;
    });

    // Clear the container and reappend the sorted panels.
    container.innerHTML = '';
    panels.forEach(panel => container.appendChild(panel));
}

function resetFilterButtons() {
    const filterButtons = document.querySelectorAll('#filters .filter-button');
    filterButtons.forEach(btn => {
        btn.dataset.sortOrder = 'asc';
        const img = btn.querySelector('img');
        if (img) {
            img.style.transform = 'rotate(0deg)';
        }
    });
}
