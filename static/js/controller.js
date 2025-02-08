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


//!!!!! CONNECTING NODES FOR TESTING for now !!!!
connections.forEach(({ from, to }) => {
    const fromNode = drawn_nodes.find((n) => n.id == from);
    const toNode = drawn_nodes.find((n) => n.id == to);

    if (fromNode && toNode) {
        const line = document.createElementNS("http://www.w3.org/2000/svg", "line");
        line.setAttribute("x1", fromNode.x);
        line.setAttribute("y1", fromNode.y);
        line.setAttribute("x2", toNode.x);
        line.setAttribute("y2", toNode.y);
        line.classList.add("connection");
        canvas.appendChild(line);
    }
});

//!!!!! DRAWING NODES FOR TESTING for now !!!!
drawn_nodes.forEach(({ id, type, x, y }) => {
    const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
    circle.setAttribute("cx", x);
    circle.setAttribute("cy", y);
    circle.setAttribute("r", 15);
    circle.classList.add("node", type);
    circle.addEventListener("click", () => alert(`Node: ${id}`));
    canvas.appendChild(circle);
});


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

        // After animation completes, hide the menu
        dropdownMenu.addEventListener(
            'animationend',
            () => {
                dropdownMenu.classList.remove('closing');
                dropdownMenu.style.display = 'none';
            },
            { once: true } // Ensure the listener runs only once
        );
    } else {
        // Start slide-in animation
        dropdownMenu.style.display = 'block'; // Ensure the content is displayed
        dropdownMenu.classList.add('active');
    }
}

// Close the dropdown if the user clicks outside
document.addEventListener('click', (event) => {
    const dropdownMenu = document.getElementById('dropdown-menu');
    const dropdownButton = document.querySelector('.dropdown button');
    if (!dropdownMenu.contains(event.target) && !dropdownButton.contains(event.target)) {
        dropdownMenu.style.display = 'none';
    }
});














/*

    CONTROLLER VIEW
    -> MONITORING (only switching sections for now)

 */

function showSection(sectionId) {
    const sections = document.querySelectorAll('.section');
    sections.forEach(section => {
        if (section.id == sectionId) {
            section.style.display = 'block';
            section.style.animation = 'slide-in 0.5s ease-out';
        } else {
            section.style.animation = 'slide-out 0.5s ease-out';
            setTimeout(() => {
                section.style.display = 'none';
            }, 500); // Match the duration of the animation
        }
    });
}
// Initialize the default view
showSection('clients-container');

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
