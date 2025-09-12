const EPOCH = new Date(Date.UTC(2025, 8, 12, 10));
const clickable = "div:not([data-val]):not(.risk-shown)";
const subtractFlagBox = document.getElementById("subtract-flags");

function statusOf(grid) {
    return grid.parentElement.getElementsByClassName("status")[0];
}

function reportOf(grid) {
    return grid.parentElement.getElementsByClassName("report")[0];
}

function drawPuzzle(grid, puzzle) {
    subtractFlagBox.onchange = () => {
        drawPuzzle(grid, puzzle);
        replay(grid, puzzle);
    };

    const report = reportOf(grid);
    report.onclick = copyReport(puzzle);
    report.classList.remove("revealed");
    report.textContent = "​";

    const status = statusOf(grid);
    status.textContent = "3 tries";

    const density = status.previousElementSibling;
    density.textContent = `${(puzzle.density*100).toFixed(0)}%`;
    density.classList.remove("low-density");
    density.classList.remove("high-density");
    density.classList.add(puzzle.density < 0.5 ? "low-density" : "high-density");

    const body = document.createElement("tbody");
    for (let y = 0; y < puzzle.height; y++) {
        const row = document.createElement("tr");
        for (let x = 0; x < puzzle.width; x++) {
            let value = puzzle.grid[y*puzzle.width+x];
            const cell = document.createElement("td");
            if (value !== undefined) {
                const div = document.createElement("div");
                if (puzzle.risks.get(`${x},${y}`) === 1) {
                    div.classList.add("risk-shown");
                    div.innerHTML = '<img src="../flag.svg" draggable="false">';
                }
                if (value) {
                    if (subtractFlagBox.checked) {
                        for (const [ax, ay] of [[x, y-1], [x+1, y-1], [x+1, y], [x+1, y+1], [x, y+1], [x-1, y+1], [x-1, y], [x-1, y-1]]) {
                            value -= puzzle.risks.get(`${ax},${ay}`) === 1;
                        }
                    }
                    div.textContent = value;
                    div.setAttribute("data-val", value.toString());
                } else {
                    div.addEventListener("click", click(puzzle));
                }
                cell.appendChild(div);
            }
            row.appendChild(cell);
        }
        body.appendChild(row);
    }
    grid.replaceChildren(body);
}

function position(div) {
    return [div.parentElement.cellIndex, div.parentElement.parentElement.rowIndex];
}

function reveal(puzzle, div) {
    const [x, y] = position(div);
    const risk = puzzle.risks.get(`${x},${y}`);
    div.textContent = `${(risk*100).toFixed(2)}`;
    div.classList.add("risk-shown");
    div.style.setProperty("--riskiness", `${risk*100}%`);
    if (risk === puzzle.sortedRisks[0]) div.classList.add("best");
    return risk;
}

function revealAll(puzzle, table) {
    reportOf(table).classList.add("revealed");
    for (const div of table.querySelectorAll(clickable)) {
        reveal(puzzle, div);
    }
}

function copyReport(puzzle) {
    return async function () {
        const day = new Date(EPOCH);
        day.setDate(day.getDate() + puzzle.num - 1);
        const report = `[insane minute minefair](<https://lyricly.github.io/minefair/insane>) #${puzzle.num} (${day.toISOString().slice(0, 10)})\n${this.textContent.slice(1)}`
        await navigator.clipboard.write([new ClipboardItem({"text/plain": report})]);
    };
}

function click(puzzle) {
    return function (event, replaying) {
        if (this.classList.contains("risk-shown")) return;
        this.classList.add("clicked");

        if (!replaying) localStorage.setItem("insane-clicks", JSON.stringify(JSON.parse(localStorage.getItem("insane-clicks")).concat([position(this)])));

        const table = this.closest("table");
        const status = statusOf(table);
        const report = reportOf(table);

        const risk = reveal(puzzle, this);
        if (puzzle.num >= 28) {
            const perfRange = (risk - puzzle.sortedRisks[1]) / (puzzle.sortedRisks[puzzle.sortedRisks.length-1] - puzzle.sortedRisks[1]);
            const perfIndex = (puzzle.sortedRisks.indexOf(risk)-1) / (puzzle.sortedRisks.length-2);
            const perf = (perfRange + perfIndex) / 2;
            report.textContent += perf < 0 ? "🟩" : perf < 1/3 ? "🟨" : perf < 2/3 ? "🟧" : "🟥";
        } else {
            const perf = risk - puzzle.sortedRisks[0];
            report.textContent += perf === 0.0 ? "🟩" : perf < 0.2 ? "🟨" : perf < 0.4 ? "🟧" : "🟥";
        }

        if (risk === puzzle.sortedRisks[0]) {
            status.textContent = "well done";
            revealAll(puzzle, table);
            return;
        }

        const statuses = ["3 tries", "2 tries", "1 try", "you tried"];
        if ((status.textContent = statuses[statuses.indexOf(status.textContent)+1]) === statuses.at(-1)) {
            revealAll(puzzle, table);
        }
    };
}

function getPuzzle(view, idx) {
    let p = 0;

    function readOff() {
        return view.getFloat32((p += 4) - 4, true);
    }

    for (let i = 0; i < idx; i++) {
        const width = readOff();
        const height = readOff();
        p += 4*(1+width*height);
    }

    const r = {
        width: readOff(),
        height: readOff(),
        density: readOff(),
        grid: [],
        risks: new Map(),
        num: idx+1,
    };

    for (let y = 0; y < r.height; y++) {
        for (let x = 0; x < r.width; x++) {
            const val = readOff();
            if (val >= 2) r.grid.push(val - 2)
            else if (val == -1) r.grid.push(undefined);
            else {
                r.grid.push(null);
                r.risks.set(`${x},${y}`, val);
            }
        }
    }

    const groups = Map.groupBy(r.risks.values(), x => x);
    groups.delete(1);
    r.sortedRisks = [...groups.keys()].sort((a, b) => a - b);

    return r;
}

function reset(grid, view, day) {
    localStorage.setItem("insane-puzzle", day.toString());
    localStorage.setItem("insane-clicks", JSON.stringify([]));
    drawPuzzle(grid, getPuzzle(view, day));
}

function replay(grid, puzzle) {
    for (const [x, y] of JSON.parse(localStorage.getItem("insane-clicks"))) {
        click(puzzle).call(grid.rows[y].cells[x].firstElementChild, null, true);
    }
}

async function main() {
    subtractFlagBox.checked = +localStorage.getItem("subtract-flags");
    subtractFlagBox.addEventListener("change", function () { localStorage.setItem("subtract-flags", +this.checked) });

    const grid = document.getElementById("grid");

    const resp = await fetch("puzzles");
    const view = new DataView(await resp.arrayBuffer());

    const current = parseInt(localStorage.getItem("insane-puzzle"), 10);
    const today = Math.floor((new Date() - EPOCH) / (24*60*60*1000));
    if (isNaN(current)) return reset(grid, view, today);

    const puzzle = getPuzzle(view, current);
    drawPuzzle(grid, puzzle);
    replay(grid, puzzle);

    if (current !== today && (
        grid.querySelector(clickable) === null
     || grid.querySelector("div:not([data-val]).risk-shown:not(:has(img))") === null
    )) {
        reset(grid, view, today);
    }

    const scaleBy = document.body.offsetWidth / grid.offsetWidth;
    const scaledOff = grid.offsetHeight * (1-scaleBy) / 2;
    grid.style.setProperty("transform", `scale(${scaleBy})`);
    grid.style.setProperty("margin", `${-scaledOff}px 0`);
}

main();
