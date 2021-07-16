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
                const speed = Number(document.getElementById('create-speed').value);
                const foods = Number(document.getElementById('create-foods').value);
                const foodStrength = Number(document.getElementById('create-food-strength').value);
                const reservedFood = document.getElementById('create-reserved-food').checked ? 1 : 0;

                const nameData = new ByteBuffer();
                nameData.implicitGrowth = true;
                const nameSize = nameData.writeString(name);

                const data = new ByteBuffer(1 + 2 + nameSize + 2 + 2 + 1 + 2 + 2 + 1);
                data.writeUnsignedByte(0);
                data.writeUnsignedShort(nameSize);
                data.write(nameData);
                data.writeUnsignedShort(width);
                data.writeUnsignedShort(height);
                data.writeUnsignedByte(speed);
                data.writeUnsignedShort(foods);
                data.writeUnsignedShort(foodStrength);
                data.writeUnsignedByte(reservedFood);
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
                this.removeGame(data);
                break;
            case 2:
                this.updatePlayerCount(data);
                break;
            case 3:
                this.joinCreated(data);
                break;
        }
    }

    addGames(data) {
        if (data.available === 0) {    
            document.getElementById('tab-create').checked = true;
            document.getElementById('create-name').focus();
        }

        while (data.available) {
            const id = data.readUnsignedShort();
            const nameLength = data.readUnsignedByte();
            const name = data.readString(nameLength);
            const size = {
                width: data.readUnsignedShort(),
                height: data.readUnsignedShort(),
            };
            const speed = data.readUnsignedByte();
            const playerCount = data.readUnsignedByte();
            this.games[String(id)] = new LobbyGame(id, { name, size, speed, playerCount });
        }
    }

    removeGame(data) {
        const id = String(data.readUnsignedShort());
        this.games[id].game.remove();
        delete this.games[id];
    }

    updatePlayerCount(data) {
        const id = String(data.readUnsignedShort());
        this.games[id].updatePlayerCount(String(data.readUnsignedByte()));
    }

    joinCreated(data) {
        const id = data.readUnsignedShort();
        new Game(id);

        document.getElementById('tab-games').checked = true;
        document.getElementById('create-name').value = '';
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
        size.classList.add('size', 'icon');
        size.innerText = `${info.size.width}x${info.size.height}`;

        const speed = document.createElement('div');
        speed.classList.add('speed', 'icon');
        speed.innerText = String(info.speed);
        
        this.players = document.createElement('div');
        this.players.classList.add('players', 'icon');
        this.players.innerText = String(info.playerCount);

        const join = document.createElement('div');
        join.classList.add('join');
        join.addEventListener('click', () => {
            new Game(id);
        });

        this.game.append(name, size, separator.cloneNode(), speed, separator.cloneNode(), this.players, separator.cloneNode(), join);
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
                this.addPerk(data);
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
        this.perks = {};

        this.canvas = document.createElement('canvas');
        this.context = this.canvas.getContext('2d');
        this.resizeHandler = (additionalHeight) => {
            const mainSize = document.getElementById('main').getBoundingClientRect();
            this.cellSize = Math.max(mainSize.width / this.size.width | 0, (mainSize.height + additionalHeight) / this.size.height | 0);
            this.cellSpacing = this.cellSize > 50 ? 2 : this.cellSize > 20 ? 1 : 0;
            this.canvas.width = this.size.width * this.cellSize + 2 * BORDER_WIDTH;
            this.canvas.height = this.size.height * this.cellSize + 2 * BORDER_WIDTH;
            
            const scale = Math.min(mainSize.width * 0.9 / this.canvas.width, (mainSize.height + additionalHeight) * 0.9 / this.canvas.height);
            this.canvas.style.width = `${this.canvas.width * scale | 0}px`;
            this.canvas.style.height = `${this.canvas.height * scale | 0}px`;
            this.redrawCanvas();
        };
        this.resizeHandler(75);
        window.addEventListener('resize', () => this.resizeHandler(0));

        const nameLength = data.readUnsignedByte();
        const name = data.readString(nameLength);
        this.selfId = data.readUnsignedShort();

        const header = document.createElement('div');
        header.classList.add('header');
        
        const title = document.createElement('div');
        title.classList.add('title');
        title.innerText = name;

        const leave = document.createElement('div');
        leave.classList.add('leave');
        leave.innerText = 'Leave';
        leave.addEventListener('click', () => {
            this.socket.close();
            document.body.classList.replace('playing', 'lobbying');
            const game = document.getElementById('game');
            while (game.firstChild) {
                game.removeChild(game.lastChild);
            }
        });

        header.append(title, leave);
        document.getElementById('game').append(header, this.canvas);
        document.body.classList.replace('lobbying', 'playing');
    }

    redrawCanvas() {
        this.drawBorders();
        this.emptyCanvas();

        for (const id in this.players) {
            const player = this.players[id];
            this.fillMode(player.color[0]);
            for (let i = 0; i < player.body.length; i++) {
                if (i === 1) {
                    this.fillMode(player.color[1]);
                }
                this.drawCell(player.body[i]);
            }
        }
        for (const coordStr in this.perks) {
            this.drawPerk(this.perks[coordStr]);
        }
    }

    drawBorders() {
        this.context.strokeStyle = 'white';
        this.context.lineWidth = BORDER_WIDTH;
        this.context.strokeRect(BORDER_WIDTH, BORDER_WIDTH, this.canvas.width - 2 * BORDER_WIDTH, this.canvas.height - 2 * BORDER_WIDTH);
    }

    emptyCanvas() { 
        this.clearMode();
        this.context.fillRect(BORDER_WIDTH, BORDER_WIDTH, this.canvas.width - 2 * BORDER_WIDTH, this.canvas.height - 2 * BORDER_WIDTH);
    }

    setPlayers(data) {
        while (data.available) {
            const id = data.readUnsignedShort();
            const color = [hslFromShort(data.readUnsignedShort()), hslFromShort(data.readUnsignedShort())];
            const size = data.readUnsignedShort();
            const body = [];
            this.fillMode(color[0]);
            for (let i = 0; i < size; i++) {
                const cell = {
                    x: data.readUnsignedShort(), 
                    y: data.readUnsignedShort(),
                };
                body.push(cell);
                if (i === 1) {
                    this.fillMode(color[1]);
                }
                this.drawCell(cell);
            }
            this.players[id] = { color, body };
        }
    }

    addPlayer(data) {
        const id = data.readUnsignedShort();
        const color = [hslFromShort(data.readUnsignedShort()), hslFromShort(data.readUnsignedShort())];
        const head = {
            x: data.readUnsignedShort(),
            y: data.readUnsignedShort(),
        };
        this.players[id] = { color, body: [head] };
        this.fillMode(color[0]);
        this.drawCell(head);
    }

    removePlayer(data) {
        const id = data.readUnsignedShort();
        this.clearMode();
        this.drawCell(this.players[id].body);
        delete this.players[id];
    }

    snakeChanges(data) {
        while (data.available) {
            switch (data.readUnsignedByte()) {
                case 0: {
                    this.clearMode();
                    this.drawCell(this.players[data.readUnsignedShort()].body.pop());
                } break;
                case 1: {
                    const player = this.players[data.readUnsignedShort()];
                    const head = {
                        x: data.readUnsignedShort(),
                        y: data.readUnsignedShort(),
                    };
                    player.body.unshift(head);
                    delete this.perks[`${head.x},${head.y}`];

                    this.fillMode(player.color[1]);
                    this.drawCell(player.body[1]);
                    this.fillMode(player.color[0]);
                    this.drawCell(head);
                } break;
                case 2: {
                    const player = this.players[data.readUnsignedShort()];
                    this.clearMode();
                    this.drawCell(player.body);
    
                    const head = {
                        x: data.readUnsignedShort(),
                        y: data.readUnsignedShort(),
                    };
                    player.body = [head];
                    this.fillMode(player.color[0]);
                    this.drawCell(head);
                } break;
            }
        }
    }

    addPerk(data) {
        const coord = {
            x: data.readUnsignedShort(),
            y: data.readUnsignedShort(),
        };
        const id = data.readUnsignedByte();
        const perk = { coord, id };
        switch (id) {
            case 1:
                perk.owner = data.readUnsignedShort();
                break;
        }
        this.perks[`${coord.x},${coord.y}`] = perk;
        this.drawPerk(perk);
    }

    clearMode() {
        this.context.fillStyle = '#000000';
    }

    fillMode(color) {
        this.context.fillStyle = color;
    }

    drawCell(coords) {
        for (const { x, y } of coords instanceof Array ? coords : [coords]) {
            this.context.fillRect(BORDER_WIDTH + x * this.cellSize + this.cellSpacing, BORDER_WIDTH + y * this.cellSize + this.cellSpacing, this.cellSize - this.cellSpacing * 2, this.cellSize - this.cellSpacing * 2);
        }
    }

    drawPerk(perk) {
        switch (perk.id) {
            case 0:
                this.context.fillStyle = '#00ff00';
                break;
            case 1:
                this.context.fillStyle = perk.owner === this.selfId ? '#1e90ff' : '#0C3B66';
                break;
            default: return;
        }
        this.context.beginPath();
        this.context.arc(BORDER_WIDTH + perk.coord.x * this.cellSize + this.cellSize / 2, BORDER_WIDTH + perk.coord.y * this.cellSize + this.cellSize / 2, this.cellSize / 4, 0, 2 * Math.PI);
        this.context.fill();
    }
}

function baseWebsocketUrl() {
    return `${location.protocol.slice(0, -1) === 'https' ? 'wss' : 'ws'}://${location.host}`;
}

function hslFromShort(color) {
    return `hsl(${color}, 100%, 50%)`;
}

function animateTitle() {
    const context = document.getElementById('title').getContext('2d');
    function fillCell(color, { x, y }) {
        context.fillStyle = color;
        context.fillRect(x * 25 + 1, y * 25 + 1, 23, 23);
    }

    const letters = [
        {
            color: 'red',
            frames: [[{ x: 2, y: 0 }], [{ x: 1, y: 0 }], [{ x: 0, y: 0 }], [{ x: 0, y: 1 }], [{ x: 0, y: 2 }], [{ x: 1, y: 2 }], [{ x: 2, y: 2 }], [{ x: 2, y: 3 }], [{ x: 2, y: 4 }], [{ x: 1, y: 4 }], [{ x: 0, y: 4 }]]
        },
        {
            color: 'green',
            frames: [[{ x: 4, y: 4 }], [{ x: 4, y: 3 }], [{ x: 4, y: 2 }], [{ x: 4, y: 1 }], [{ x: 4, y: 0 }], [{ x: 5, y: 0 }], [{ x: 6, y: 0 }], [{ x: 6, y: 0 }], [{ x: 6, y: 1 }], [{ x: 6, y: 2 }], [{ x: 6, y: 3 }], [{ x: 6, y: 4 }]]
        },
        {
            color: 'purple',
            frames: [[{ x: 8, y: 4 }], [{ x: 8, y: 3 }], [{ x: 8, y: 2 }], [{ x: 8, y: 1 }], [{ x: 8, y: 0 }], [{ x: 9, y: 0 }], [{ x: 10, y: 0 }], [{ x: 10, y: 0 }], [{ x: 10, y: 1 }], [{ x: 10, y: 2 }, { x: 9, y: 2 }], [{ x: 10, y: 3 }], [{ x: 10, y: 4 }]]
        },
        {
            color: 'blue',
            frames: [[{ x: 12, y: 0 }], [{ x: 12, y: 1 }], [{ x: 12, y: 2 }], [{ x: 12, y: 3 }, { x: 13, y: 2 }], [{ x: 12, y: 4 }, { x: 14, y: 1 }, { x: 14, y: 3 }], [{ x: 14, y: 0 }, { x: 14, y: 4 }]]
        },
        {
            color: 'orange',
            frames: [[{ x: 18, y: 0 }], [{ x: 17, y: 0 }], [{ x: 16, y: 0 }], [{ x: 16, y: 1 }], [{ x: 16, y: 2 }], [{ x: 16, y: 3 }, { x: 17, y: 2 }], [{ x: 16, y: 4 }], [{ x: 17, y: 4 }], [{ x: 18, y: 4 }]]
        }
    ];

    for (const letter of letters) {
        for (let i = 0; i < letter.frames.length; i++) {
            setTimeout(() => {
                for (const cell of letter.frames[i]) {
                    fillCell(letter.color, cell);
                }
            }, i * 100);
        }
    }
}

document.querySelectorAll('.validable').forEach((elem) => {
    function setProcessButtonState() {
        document.querySelector('#lobby > .create > .content > .actions > .process').classList.toggle('disabled', !Array.from(document.querySelectorAll('.validable')).every((elem) => elem.checkValidity()));
    }
    elem.addEventListener('change', setProcessButtonState);
    elem.addEventListener('keyup', setProcessButtonState);
});

document.getElementById('tab-create').addEventListener('change', () => {
    document.getElementById('create-name').focus();
});

new Lobby();
animateTitle();