// chartWorker.js
import { WebglPlot, WebglLine, ColorRGBA } from "webgl-plot";

let plotCanvas, xAxisCanvas, yAxisCanvas;
let wglp = null;
let lines = [];
let windowSize = 1000;
let yMin = -1;
let yMax = 1.05;
let dataBuffer = [];

// Helper: initialize a 2D context on an OffscreenCanvas.
function initAxisCanvas(canvas) {
  if (!canvas) return null;
  const ctx2d = canvas.getContext("2d");
  ctx2d.font = "12px Arial";
  ctx2d.fillStyle = "#4a5568";
  ctx2d.strokeStyle = "#cbd5e0";
  ctx2d.lineWidth = 1;
  return ctx2d;
}

function updateYAxis() {
  if (!yAxisCanvas) return;
  const ctx2d = initAxisCanvas(yAxisCanvas);
  if (!ctx2d) return;
  const width = yAxisCanvas.width;
  const height = yAxisCanvas.height;
  const divisions = 8;
  ctx2d.clearRect(0, 0, width, height);
  for (let i = 0; i <= divisions; i++) {
    const value = yMax - (i / divisions) * (yMax - yMin);
    const y = (i / divisions) * (height - 20);
    ctx2d.beginPath();
    ctx2d.moveTo(width - 5, y);
    ctx2d.lineTo(width, y);
    ctx2d.stroke();
    ctx2d.fillText(value.toFixed(1), 2, y + 4);
  }
}

function updateXAxis() {
  if (!xAxisCanvas) return;
  const ctx2d = initAxisCanvas(xAxisCanvas);
  if (!ctx2d) return;
  const width = xAxisCanvas.width;
  const height = xAxisCanvas.height;
  const divisions = 10;
  ctx2d.clearRect(0, 0, width, height);
  for (let i = 0; i <= divisions; i++) {
    const position = (i / divisions) * width;
    ctx2d.beginPath();
    ctx2d.moveTo(position, 0);
    ctx2d.lineTo(position, 5);
    ctx2d.stroke();
    // For simplicity, the label is computed from the window size.
    ctx2d.fillText(Math.floor((i / divisions) * windowSize).toString(), position - 10, 20);
  }
}

function initPlot() {
  if (!plotCanvas) return;
  wglp = new WebglPlot(plotCanvas);
  lines = [];
  const channelColors = [
    "#FF6384",
    "#36A2EB",
    "#FFCE56",
    "#4BC0C0",
    "#9966FF",
    "#FF9F40",
    "#E7E9ED",
    "#7CFFC4",
  ];

  for (let i = 0; i < 8; i++) {
    // Convert hex color to normalized RGBA.
    const color = new ColorRGBA(
      parseInt(channelColors[i].substr(1, 2), 16) / 255,
      parseInt(channelColors[i].substr(3, 2), 16) / 255,
      parseInt(channelColors[i].substr(5, 2), 16) / 255,
      1
    );
    const line = new WebglLine(color, windowSize);
    line.arrangeX();
    wglp.addLine(line);
    lines.push(line);
  }
  updateYAxis();
  updateXAxis();
  animate();
}

// Clear plot and reinitialize.
function clearPlot() {
  dataBuffer = [];
  wglp = null;
  initPlot();
}

// This example implements a simple new‑data update.
// In a real implementation you would update only the parts of the plot that changed.
function addNewDataPoints(newData) {
  if (!newData || !lines.length) return;
  // For each channel, shift‑add the new data.
  for (let ch = 0; ch < 8 && ch < lines.length; ch++) {
    newData.forEach((dataPoint) => {
      lines[ch].shiftAdd(new Float32Array([dataPoint[ch]]));
    });
  }
}

// Animation loop to update the WebGL‑plot.
function animate() {
  if (wglp) {
    wglp.update();
  }
  requestAnimationFrame(animate);
}

// --- Message Handler: receive messages from the main thread ---
onmessage = function (event) {
  const data = event.data;
  switch (data.type) {
    case "init":
      // Save configuration and OffscreenCanvases.
      windowSize = data.config.windowSize || windowSize;
      yMin = data.config.yMin || yMin;
      yMax = data.config.yMax || yMax;
      plotCanvas = data.canvases.plot;
      xAxisCanvas = data.canvases.xAxis;
      yAxisCanvas = data.canvases.yAxis;
      initPlot();
      break;
    case "update-config":
      if (data.config.windowSize !== undefined) {
        windowSize = data.config.windowSize;
        initPlot();
      }
      if (data.config.yMin !== undefined) {
        yMin = data.config.yMin;
        updateYAxis();
      }
      if (data.config.yMax !== undefined) {
        yMax = data.config.yMax;
        updateYAxis();
      }
      break;
    case "clear-plot":
      clearPlot();
      break;
    case "new-data":
      addNewDataPoints(data.newData);
      break;
    case "resize":
      updateXAxis();
      updateYAxis();
      break;
    default:
      console.warn("Unknown message type in worker:", data.type);
  }
};
