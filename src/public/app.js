const BORDER_WIDTH = 5;

class Lobby {
    constructor() {
        this.games = {};

        this.socket = new WebSocket(`${baseWebsocketUrl()}/lobby`);
        this.socket.binaryType = 'arraybuffer';
        this.socket.addEventListener('open', () => {
            this.socket.addEventListener('message', (event) => {
                this.processMessage(new ByteBuffer(event.data));
            });

            document.getElementById('create-join').addEventListener('click', () => {
                const name = document.getElementById('create-name').value;
                const width = Number(document.getElementById('create-width').value);
                const height = Number(document.getElementById('create-height').value);
                const foods = Number(document.getElementById('create-foods').value);
                const food_strength = Number(document.getElementById('create-food-strength').value);

                const nameData = new ByteBuffer();
                nameData.implicitGrowth = true;
                const nameSize = nameData.writeString(name);

                const data = new ByteBuffer(1 + 2 + nameSize + 2 + 2 + 2 + 2);
                data.writeUnsignedByte(0);
                data.writeUnsignedShort(nameSize);
                data.write(nameData);
                data.writeUnsignedShort(width);
                data.writeUnsignedShort(height);
                data.writeUnsignedShort(foods);
                data.writeUnsignedShort(food_strength);
                this.socket.send(data.buffer);
            });
        });
    }

    processMessage(data) {
        switch (data.readUnsignedByte()) {
            case 0:
                this.addGames(data);
                break;
            case 1:
                this.updatePlayerCount(data);
                break;
            case 2:
                this.joinCreated(data);
                break;
        }
    }

    addGames(data) {
        while (data.available) {
            const id = data.readUnsignedShort();
            const nameLength = data.readUnsignedByte();
            const name = data.readString(nameLength);
            const size = {
                width: data.readUnsignedShort(),
                height: data.readUnsignedShort(),
            };
            const playerCount = data.readUnsignedByte();
            this.games[String(id)] = new LobbyGame(id, { name, size, playerCount });
        }
    }

    updatePlayerCount(data) {
        const id = String(data.readUnsignedShort());
        this.games[id].updatePlayerCount(String(data.readUnsignedByte()));
    }

    joinCreated(data) {
        const id = data.readUnsignedShort();
        new Game(id);
    }
}

class LobbyGame {
    constructor(id, info) {
        this.game = document.createElement('div');
        this.game.classList.add('game');

        const name = document.createElement('div');
        name.classList.add('name');
        name.innerText = info.name;

        const separator = document.createElement('div');
        separator.classList.add('separator');

        const size = document.createElement('div');
        size.classList.add('size');
        size.innerText = `${info.size.width}x${info.size.height}`;
        
        this.players = document.createElement('div');
        this.players.classList.add('players');
        this.players.innerText = String(info.playerCount);

        const join = document.createElement('div');
        join.classList.add('join');
        join.addEventListener('click', () => {
            new Game(id);
        });

        this.game.append(name, size, separator.cloneNode(), this.players, separator.cloneNode(), join);
        document.querySelector('#lobby > .games > .content').append(this.game);
    }

    updatePlayerCount(playerCount) {
        this.players.innerText = String(playerCount);
    }
}

class Game {
    constructor(id) {
        this.socket = new WebSocket(`${baseWebsocketUrl()}/games/${id}`);
        this.socket.binaryType = 'arraybuffer';
        this.socket.addEventListener('open', () => {
            this.socket.addEventListener('message', (event) => {
                this.processMessage(new ByteBuffer(event.data));
            });

            this.keyEventHandler = (event) => {
                this.processKey(event);
            }
            window.addEventListener('keydown', this.keyEventHandler);
        });
    }

    processMessage(data) {
        switch (data.readUnsignedByte()) {
            case 0:
                this.create(data);
                break;
            case 1:
                this.setPlayers(data);
                break;
            case 2:
                this.drawPerk(data);
                break;
            case 3:
                this.addPlayer(data);
                break;
            case 4:
                this.removePlayer(data);
                break;
            case 5:
                this.snakeChanges(data);
                break;
        }
    }

    processKey(event) {
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
        this.socket.send(new Uint8Array([0, key]));
    }

    create(data) {
        this.size = {
            width: data.readUnsignedShort(),
            height: data.readUnsignedShort(),
        };
        this.players = {};
        this.cellSize = Math.max(window.innerWidth * 0.9 / this.size.width | 0, window.innerHeight * 0.9 / this.size.height | 0);
        this.cellSpacing = this.cellSize > 50 ? 2 : this.cellSize > 20 ? 1 : 0;
        this.canvas = document.createElement('canvas');
        this.canvas.width = this.size.width * this.cellSize + 2 * BORDER_WIDTH;
        this.canvas.height = this.size.height * this.cellSize + 2 * BORDER_WIDTH;
        this.context = this.canvas.getContext('2d');
        this.emptyCanvas();
        document.getElementById('game').append(this.canvas);
    }

    emptyCanvas() {
        this.context.strokeStyle = 'white';
        this.context.lineWidth = BORDER_WIDTH;
        this.context.strokeRect(BORDER_WIDTH, BORDER_WIDTH, this.canvas.width - 2 * BORDER_WIDTH, this.canvas.height - 2 * BORDER_WIDTH);
        this.clearMode();
        this.context.fillRect(BORDER_WIDTH, BORDER_WIDTH, this.canvas.width - 2 * BORDER_WIDTH, this.canvas.height - 2 * BORDER_WIDTH);
    }

    setPlayers(data) {
        this.fillMode();
        while (data.available) {
            const id = data.readUnsignedShort();
            const size = data.readUnsignedShort();
            const body = [];
            for (let j = 0; j < size; j++) {
                const cell = {
                    x: data.readUnsignedShort(), 
                    y: data.readUnsignedShort(),
                };
                body.push(cell);
                this.drawCell(cell);
            }
            this.players[id] = body;
        }
    }

    addPlayer(data) {
        const id = data.readUnsignedShort();
        const head = {
            x: data.readUnsignedShort(),
            y: data.readUnsignedShort(),
        };
        this.players[id] = [head];
        this.fillMode();
        this.drawCell(head);
    }

    removePlayer(data) {
        const id = data.readUnsignedShort();
        this.clearMode();
        this.drawCell(this.players[id]);
        delete this.players[id];
    }

    snakeChanges(data) {
        while (data.available) {
            switch (data.readUnsignedByte()) {
                case 0: {
                    this.clearMode();
                    this.drawCell(this.players[data.readUnsignedShort()].pop());
                } break;
                case 1: {
                    const id = data.readUnsignedShort();
                    const head = {
                        x: data.readUnsignedShort(),
                        y: data.readUnsignedShort(),
                    };
                    this.players[id].unshift(head);
                    this.fillMode();
                    this.drawCell(head);
                } break;
                case 2: {
                    const id = data.readUnsignedShort();
                    this.clearMode();
                    this.drawCell(this.players[id]);
    
                    const head = {
                        x: data.readUnsignedShort(),
                        y: data.readUnsignedShort(),
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
            this.context.fillRect(BORDER_WIDTH + x * this.cellSize + this.cellSpacing, BORDER_WIDTH + y * this.cellSize + this.cellSpacing, this.cellSize - this.cellSpacing * 2, this.cellSize - this.cellSpacing * 2);
        }
    }

    drawPerk(data) {
        this.context.fillStyle = 'red';
        this.context.beginPath();
        this.context.arc(BORDER_WIDTH + data.readUnsignedShort() * this.cellSize + this.cellSize / 2, BORDER_WIDTH + data.readUnsignedShort() * this.cellSize + this.cellSize / 2, this.cellSize / 4, 0, 2 * Math.PI);
        this.context.fill();
    }
}

function baseWebsocketUrl() {
    return `${location.protocol.slice(0, -1) === 'https' ? 'wss' : 'ws'}://${location.host}`;
}

document.querySelectorAll('.validable').forEach((elem) => {
    function setProcessButtonState() {
        document.querySelector('#lobby > .create > .content > .actions > .process').classList.toggle('disabled', !Array.from(document.querySelectorAll('.validable')).every((elem) => elem.checkValidity()));
    }
    elem.addEventListener('change', setProcessButtonState);
    elem.addEventListener('keyup', setProcessButtonState);
});

new Lobby();