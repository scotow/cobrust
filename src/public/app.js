const BORDER_WIDTH = 5;
const CELL_SIZE = 50;

let currentGame = null;

class Game {
    constructor(size) {
        this.size = size;
        this.players = {};
        this.canvas = document.createElement('canvas');
        this.canvas.width = size.width * CELL_SIZE + 2 * BORDER_WIDTH;
        this.canvas.height = size.height * CELL_SIZE + 2 * BORDER_WIDTH;
        this.context = this.canvas.getContext('2d');
        this.emptyCanvas();
        document.getElementById('game').append(this.canvas);
    }

    emptyCanvas() {
        this.context.strokeStyle = 'white';
        this.context.lineWidth = BORDER_WIDTH;
        this.context.strokeRect(0, 0, this.canvas.width, this.canvas.height);
        this.clearMode();
        this.context.fillRect(BORDER_WIDTH, BORDER_WIDTH, this.canvas.width - 2 * BORDER_WIDTH, this.canvas.height - 2 * BORDER_WIDTH);
    }

    setPlayers(data) {
        this.fillMode();
        let i = 0;
        while (i < data.length) {
            const id = data[i++];
            const size = data[i++];
            const body = [];
            for (let j = 0; j < size; j++) {
                const cell = {
                    x: data[i++], 
                    y: data[i++],
                };
                body.push(cell);
                this.drawCell(cell);
            }
            this.players[id] = body;
        }
    }

    addPlayer(data) {
        const head = {
            x: data[1],
            y: data[2],
        };
        this.players[data[0]] = [head];
        this.fillMode();
        this.drawCell(head);
    }

    removePlayer(data) {
        this.clearMode();
        this.drawCell(this.players[data[0]]);
        delete this.players[data[0]];
    }

    snakeChanges(data) {
        let i = 0;
        while (i < data.length) {
            switch (data[i++]) {
                case 0: {
                    this.clearMode();
                    this.drawCell(this.players[data[i++]].pop());
                } break;
                case 1: {
                    const id = data[i++];
                    const head = {
                        x: data[i++],
                        y: data[i++],
                    };
                    this.players[id].unshift(head);
                    this.fillMode();
                    this.drawCell(head);
                } break;
                case 2: {
                    const id = data[i++];
                    this.clearMode();
                    this.drawCell(this.players[id]);
    
                    const head = {
                        x: data[i++],
                        y: data[i++],
                    };
                    this.players[id] = [head];
                    this.fillMode();
                    this.drawCell(head);
                } break;
            }
        }
    }

    clearMode() {
        this.context.fillStyle = 'black';
    }

    fillMode() {
        this.context.fillStyle = 'white';
    }

    drawCell(coords) {
        for (const { x, y } of coords instanceof Array ? coords : [coords]) {
            this.context.fillRect(BORDER_WIDTH + x * CELL_SIZE + 2, BORDER_WIDTH + y * CELL_SIZE + 2, CELL_SIZE - 4, CELL_SIZE - 4);
        }
    }

    drawPerk(data) {
        this.context.fillStyle = 'red';
        this.context.beginPath();
        this.context.arc(BORDER_WIDTH + data[0] * CELL_SIZE + CELL_SIZE / 2, BORDER_WIDTH + data[1] * CELL_SIZE + CELL_SIZE / 2, CELL_SIZE / 4, 0, 2 * Math.PI);
        this.context.fill();
    }
}
document.querySelector('#lobby > .games').addEventListener('click', (event) => {
    if (event.target.classList.contains('join')) {
        const socket = new WebSocket(`${location.protocol.slice(0, -1) === 'https' ? 'wss' : 'ws'}://${location.host}/ws`);
        socket.binaryType = 'arraybuffer';
        socket.addEventListener('open', () => {
            socketReady(socket);
        });
    }
});

function socketReady(socket) {
    socket.addEventListener('message', function (event) {
        const payload = new Uint16Array(event.data.slice(1));
        switch (new Uint8Array(event.data, 0, 1)[0]) {
            case 0:
                currentGame = new Game({
                    width: payload[0],
                    height: payload[1],
                });
                break;
            case 1:
                currentGame.setPlayers(payload);
                break;
            case 2:
                currentGame.drawPerk(payload);
                break;
            case 3:
                currentGame.addPlayer(payload);
                break;
            case 4:
                currentGame.removePlayer(payload);
                break;
            case 5:
                currentGame.snakeChanges(payload);
                break;
        }
    });

    window.addEventListener('keydown', (event) => {
        let key;
        switch (event.code) {
            case 'ArrowUp':
            case 'KeyW':
                key = 0;
                break;
            case 'ArrowDown':
            case 'KeyS':
                key = 1;
                break;
            case 'ArrowLeft':
            case 'KeyA':
                key = 2;
                break;
            case 'ArrowRight':
            case 'KeyD':
                key = 3;
                break;
            default:
                return;
        }
        socket.send(new Uint8Array([1, key]));
    });
}