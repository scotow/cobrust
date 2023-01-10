const BORDER_WIDTH = 5;

class Lobby {
    constructor() {
        this.games = {};
        this.setupEvents();

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
                const reverser = document.getElementById('create-reverser').checked ? 1 : 0;
                const teleporter = document.getElementById('create-teleporter').checked ? 1 : 0;
                const speedBoost = document.getElementById('create-speed-boost').checked ? Number(document.getElementById('create-speed-boost-duration').value) : 0;
                const perkSpacing = Number(document.getElementById('create-perk-spacing').value);

                const nameData = new ByteBuffer();
                nameData.implicitGrowth = true;
                const nameSize = nameData.writeString(name);

                const data = new ByteBuffer(0, ByteBuffer.BIG_ENDIAN, true);
                data.writeUnsignedByte(0);
                data.writeUnsignedShort(nameSize);
                data.write(nameData);
                data.writeUnsignedShort(width);
                data.writeUnsignedShort(height);
                data.writeUnsignedByte(speed);
                data.writeUnsignedShort(foods);
                data.writeUnsignedShort(foodStrength);
                data.writeUnsignedByte(reservedFood);
                data.writeUnsignedByte(reverser);
                data.writeUnsignedByte(teleporter);
                data.writeUnsignedShort(speedBoost);
                data.writeUnsignedShort(perkSpacing);
                this.socket.send(data.buffer);
            });
        });
    }

    setupEvents() {
        function updateForm() {
            document.getElementById('create-speed-boost-duration-group').classList.toggle('hidden', !document.getElementById('create-speed-boost').checked);
            document.querySelector('#lobby > .create > .content > .actions > .process').classList.toggle('disabled', !Array.from(document.querySelectorAll('.input:not(.hidden) > .validable')).every((elem) => elem.checkValidity()));
        }

        function createTabSelected() {
            const nameInput = document.getElementById('create-name');
            if (!nameInput.value) {
                nameInput.value = `Game ${1000 + Math.floor(Math.random() * 8999)}`;
                nameInput.select();
            }
            nameInput.focus();
            updateForm();
        }

        document.querySelectorAll('.validable').forEach((elem) => {
            elem.addEventListener('change', updateForm);
            elem.addEventListener('keyup', updateForm);
        });

        document.querySelectorAll('.toggle-other').forEach((elem) => {
            elem.addEventListener('change', updateForm);
        });

        document.getElementById('tab-create').addEventListener('change', createTabSelected);

        document.querySelector('#lobby > .games > .content').addEventListener('click', () => {
            if (Object.keys(this.games).length === 0) {
                document.getElementById('tab-create').checked = true;
                createTabSelected();
            }
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
        this.game.addEventListener('dblclick', () => {
            new Game(id);
        });

        const name = document.createElement('div');
        name.classList.add('name');
        name.innerText = info.name;

        const separator = document.createElement('div');
        separator.classList.add('separator');

        const size = document.createElement('div');
        size.classList.add('size', 'icon');
        size.title = 'Grid size';
        size.innerText = `${info.size.width}x${info.size.height}`;

        const speed = document.createElement('div');
        speed.classList.add('speed', 'icon');
        speed.title = 'Snakes speed';
        speed.innerText = String(info.speed);

        this.players = document.createElement('div');
        this.players.classList.add('players', 'icon');
        this.players.title = 'Conntected players';
        this.players.innerText = String(info.playerCount);

        const join = document.createElement('div');
        join.classList.add('join');
        join.title = 'Join game';
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
            };
            this.swipeStartEventHandler = (event) => {
                this.touch = { x: event.touches[0].clientX, y: event.touches[0].clientY };
            };
            this.swipeEndEventHandler = (event) => {
                this.processSwipe(
                    event.changedTouches[0].clientX - this.touch.x,
                    event.changedTouches[0].clientY - this.touch.y,
                );
            };

            window.addEventListener('keydown', this.keyEventHandler);
            window.addEventListener('touchstart', this.swipeStartEventHandler);
            window.addEventListener('touchend', this.swipeEndEventHandler);
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
                this.addPerks(data);
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

    processSwipe(x, y) {
        this.socket.send(new Uint8Array([0, Math.abs(x) > Math.abs(y) ? (x < 0 ? 2 : 3) : (y < 0 ? 0 : 1)]));
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
            if (typeof additionalHeight !== "number") {
                additionalHeight = 0;
            }

            const mainSize = document.getElementById('main').getBoundingClientRect();
            this.cellSize = Math.max(mainSize.width / this.size.width | 0, (mainSize.height + additionalHeight) / this.size.height | 0);
            this.cellSpacing = this.cellSize > 50 ? 2 : this.cellSize > 20 ? 1 : 0;
            this.canvas.width = this.size.width * this.cellSize + 2 * BORDER_WIDTH;
            this.canvas.height = this.size.height * this.cellSize + 2 * BORDER_WIDTH;

            const scale = Math.min((mainSize.width - 60) / this.canvas.width, (mainSize.height + additionalHeight - 27 - 60) / this.canvas.height);
            this.canvas.style.width = `${this.canvas.width * scale | 0}px`;
            this.canvas.style.height = `${this.canvas.height * scale | 0}px`;
            this.redrawCanvas();
        };
        this.resizeHandler(87);
        window.addEventListener('resize', this.resizeHandler);

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
            window.removeEventListener('resize', this.resizeHandler);
            window.removeEventListener('keydown', this.keyEventHandler);
            window.removeEventListener('touchstart', this.swipeStartEventHandler);
            window.removeEventListener('touchend', this.swipeEndEventHandler);

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
                case 3: {
                    const player = this.players[data.readUnsignedShort()];
                    this.fillMode(player.color[1]);
                    this.drawCell(player.body[0]);
                    player.body = player.body.reverse();
                    this.fillMode(player.color[0]);
                    this.drawCell(player.body[0]);
                } break;
            }
        }
    }

    addPerks(data) {
        while (data.available) {
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
                this.context.fillStyle = '#2fbf71';
                break;
            case 1:
                this.context.fillStyle = perk.owner === this.selfId ? '#1e90ff' : '#0c3b66';
                break;
            case 2:
                this.context.fillStyle = '#f0c808';
                break;
            case 3:
                this.context.fillStyle = '#e7820e';
                break;
            case 4:
                this.context.fillStyle = '#e70ed9';
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
    function fillCell(color, { x, y }, shift) {
        context.fillStyle = color;
        context.fillRect(x * 25 + 1, y * 25 + 1 + (shift ? 12 : 0), 23, 23);
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

    for (let l = 0; l < letters.length; l++) {
        for (let i = 0; i < letters[l].frames.length; i++) {
            setTimeout(() => {
                for (const cell of letters[l].frames[i]) {
                    fillCell(letters[l].color, cell, l % 2 === 1);
                }
            }, i * 100);
        }
    }
}

new Lobby();
animateTitle();