const SPRITE_LENGTH = 16;
const FRAME_SIZE = 64;
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
                const foodFrenzy = document.getElementById('create-food-frenzy').checked ? Number(document.getElementById('create-food-frenzy-count').value) : 0;
                const minesTrail = document.getElementById('create-mines-trail').checked ? Number(document.getElementById('create-mines-trail-count').value) : 0;
                const perkSpacing = document.getElementById('create-perk-spacing-group').classList.contains('hidden') ? 1 : Number(document.getElementById('create-perk-spacing').value);

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
                data.writeUnsignedByte(foodFrenzy);
                data.writeUnsignedByte(minesTrail);
                data.writeUnsignedShort(perkSpacing);
                this.socket.send(data.buffer);
            });
        });
    }

    setupEvents() {
        function updateForm() {
            document.getElementById('create-perk-spacing-group').classList.toggle('hidden', Array.from(document.querySelectorAll('input[type=checkbox].perk')).every(perk => !perk.checked));
            document.getElementById('create-speed-boost-duration-group').classList.toggle('hidden', !document.getElementById('create-speed-boost').checked);
            document.getElementById('create-food-frenzy-count-group').classList.toggle('hidden', !document.getElementById('create-food-frenzy').checked);
            document.getElementById('create-mines-trail-count-group').classList.toggle('hidden', !document.getElementById('create-mines-trail').checked);
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

        document.querySelectorAll('.perk').forEach((elem) => {
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
            for (let i = 0; i < player.body.length; i++) {
                this.drawFrame(player, i);
            }
        }
        for (const coordStr in this.perks) {
            this.drawPerk(this.perks[coordStr]);
        }
    }

    drawBorders() {
        this.context.strokeStyle = '#ffffff';
        this.context.lineWidth = BORDER_WIDTH;
        this.context.strokeRect(BORDER_WIDTH, BORDER_WIDTH, this.canvas.width - 2 * BORDER_WIDTH, this.canvas.height - 2 * BORDER_WIDTH);
    }

    emptyCanvas() {
        this.context.fillStyle = '#000000';
        this.context.fillRect(BORDER_WIDTH, BORDER_WIDTH, this.canvas.width - 2 * BORDER_WIDTH, this.canvas.height - 2 * BORDER_WIDTH);
    }

    setPlayers(data) {
        while (data.available) {
            const id = data.readUnsignedShort();
            const color = data.readUnsignedShort();
            const size = data.readUnsignedShort();
            const body = [];
            for (let i = 0; i < size; i++) {
                const cell = {
                    x: data.readUnsignedShort(),
                    y: data.readUnsignedShort(),
                };
                body.push(cell);
            }
            this.players[id] = { color, body, frames: generateFrames(color) };
            for (let i = 0; i < body.length; i++) {
                this.drawFrame(this.players[id], i);
            }
        }
    }

    addPlayer(data) {
        const id = data.readUnsignedShort();
        const color = data.readUnsignedShort();
        const head = {
            x: data.readUnsignedShort(),
            y: data.readUnsignedShort(),
        };
        this.players[id] = { color, body: [head], frames: generateFrames(color) };
        this.drawFrame(this.players[id], 0);
    }

    removePlayer(data) {
        const id = data.readUnsignedShort();
        this.clearCell(this.players[id].body);
        delete this.players[id];
    }

    snakeChanges(data) {
        while (data.available) {
            switch (data.readUnsignedByte()) {
                case 0: {
                    const player = this.players[data.readUnsignedShort()];
                    this.clearCell(player.body.pop());
                    this.drawFrame(player, player.body.length - 1);
                } break;
                case 1: {
                    const player = this.players[data.readUnsignedShort()];
                    const head = {
                        x: data.readUnsignedShort(),
                        y: data.readUnsignedShort(),
                    };
                    player.body.unshift(head);
                    delete this.perks[`${head.x},${head.y}`];

                    if (player.body.length >= 2) {
                        this.drawFrame(player, 1);
                    }
                    this.drawFrame(player, 0);
                } break;
                case 2: {
                    const player = this.players[data.readUnsignedShort()];
                    this.clearCell(player.body);

                    const head = {
                        x: data.readUnsignedShort(),
                        y: data.readUnsignedShort(),
                    };
                    player.body = [head];
                    this.drawFrame(player, 0);
                } break;
                case 3: {
                    const player = this.players[data.readUnsignedShort()];
                    player.body = player.body.reverse();
                    this.drawFrame(player, 0);
                    this.drawFrame(player, player.body.length - 1);
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
                case 7:
                    perk.owner = data.readUnsignedShort();
                    break;
            }
            this.perks[`${coord.x},${coord.y}`] = perk;
            this.drawPerk(perk);
        }
    }

    clearCell(coords) {
        this.context.fillStyle = '#000000';
        for (const { x, y } of coords instanceof Array ? coords : [coords]) {
            this.context.fillRect(BORDER_WIDTH + x * this.cellSize, BORDER_WIDTH + y * this.cellSize, this.cellSize, this.cellSize);
        }
    }

    drawFrame(player, index) {
        const forw = player.body[index - 1] ?? null;
        const curr = player.body[index] ?? null;
        const back = player.body[index + 1] ?? null;

        let dx1 = null, dy1 = null, dx2 = null, dy2 = null;
        if (forw !== null) {
            if (forw.x - curr.x === -1 || forw.x - curr.x === this.size.width - 1) {
                dx1 = -1;
            } else if (forw.x - curr.x === 1 || forw.x - curr.x === -this.size.width + 1) {
                dx1 = 1;
            } else {
                dx1 = forw.x - curr.x;
            }
            if (forw.y - curr.y === -1 || forw.y - curr.y === this.size.height - 1) {
                dy1 = -1;
            } else if (forw.y - curr.y === 1 || forw.y - curr.y === -this.size.height + 1) {
                dy1 = 1;
            } else {
                dy1 = forw.y - curr.y;
            }
        }
        if (back !== null) {
            if (curr.x - back.x === -1 || curr.x - back.x === this.size.width - 1) {
                dx2 = -1;
            } else if (curr.x - back.x === 1 || curr.x - back.x === -this.size.width + 1) {
                dx2 = 1;
            } else {
                dx2 = curr.x - back.x;
            }
            if (curr.y - back.y === -1 || curr.y - back.y === this.size.height - 1) {
                dy2 = -1;
            } else if (curr.y - back.y === 1 || curr.y - back.y === -this.size.height + 1) {
                dy2 = 1;
            } else {
                dy2 = curr.y - back.y;
            }
        }

        let frameIndex = 15;
        if (forw !== null && back != null && (Math.abs(dx1) >= 2 || Math.abs(dy1) >= 2)) { // Teleported.
            if (curr.x === back.x) {
                if (curr.y - back.y === -1 || curr.y - back.y === this.size.height - 1) {
                    frameIndex = 8;
                } else if (curr.y - back.y === 1 || curr.y - back.y === -this.size.height + 1) {
                    frameIndex = 7;
                }
            } else if (curr.y === back.y) {
                if (curr.x - back.x === -1 || curr.x - back.x === this.size.width - 1) {
                    frameIndex = 10;
                } else if (curr.x - back.x === 1 || curr.x - back.x === -this.size.width + 1) {
                    frameIndex = 9;
                }
            }
        } else if (forw !== null && back != null && (Math.abs(dx2) >= 2 || Math.abs(dy2) >= 2)) {
            if (forw.x === curr.x) {
                if (forw.y - curr.y === -1 || forw.y - curr.y === this.size.height - 1) {
                    frameIndex = 7;
                } else if (forw.y - curr.y === 1 || forw.y - curr.y === -this.size.height + 1) {
                    frameIndex = 8;
                }
            } else if (forw.y === curr.y) {
                if (forw.x - curr.x === -1 || forw.x - curr.x === this.size.width - 1) {
                    frameIndex = 9;
                } else if (forw.x - curr.x === 1 || forw.x - curr.x === -this.size.width + 1) {
                    frameIndex = 10;
                }
            }
        } else if (forw === null && back === null) { // Egg / spawn.
            frameIndex = 0;
        } else {
            if (forw === null) { // Head.
                if (curr.x === back.x) {
                    if (curr.y - back.y === -1 || curr.y - back.y === this.size.height - 1) {
                        frameIndex = 11;
                    } else if (curr.y - back.y === 1 || curr.y - back.y === -this.size.height + 1) {
                        frameIndex = 12;
                    }
                } else if (curr.y === back.y) {
                    if (curr.x - back.x === -1 || curr.x - back.x === this.size.width - 1) {
                        frameIndex = 13;
                    } else if (curr.x - back.x === 1 || curr.x - back.x === -this.size.width + 1) {
                        frameIndex = 14;
                    }
                }
            } else if (back === null) { // Tail.
                if (forw.x === curr.x) {
                    if (forw.y - curr.y === -1 || forw.y - curr.y === this.size.height - 1) {
                        frameIndex = 7;
                    } else if (forw.y - curr.y === 1 || forw.y - curr.y === -this.size.height + 1) {
                        frameIndex = 8;
                    }
                } else if (forw.y === curr.y) {
                    if (forw.x - curr.x === -1 || forw.x - curr.x === this.size.width - 1) {
                        frameIndex = 9;
                    } else if (forw.x - curr.x === 1 || forw.x - curr.x === -this.size.width + 1) {
                        frameIndex = 10;
                    }
                }
            } else { // Turns.
                if (forw.x === curr.x && curr.x === back.x) {
                    frameIndex = 5;
                } else if (forw.y === curr.y && curr.y === back.y) {
                    frameIndex = 6;
                } else if (forw.x === curr.x && (forw.y - curr.y === -1 || forw.y - curr.y === this.size.height - 1) && curr.y === back.y && (curr.x - back.x === -1 || curr.x - back.x === this.size.width - 1)
                    || forw.y === curr.y && (forw.x - curr.x === 1 || forw.x - curr.x === -this.size.width + 1) && curr.x === back.x && (curr.y - back.y === 1 || curr.y - back.y === -this.size.height + 1)
                ) {
                    frameIndex = 1;
                } else if (forw.x === curr.x && (forw.y - curr.y === -1 || forw.y - curr.y === this.size.height - 1) && curr.y === back.y && (curr.x - back.x === 1 || curr.x - back.x === -this.size.width + 1)
                    || forw.y === curr.y && (forw.x - curr.x === -1 || forw.x - curr.x === this.size.width - 1) && curr.x === back.x && (curr.y - back.y === 1 || curr.y - back.y === -this.size.height + 1)
                ) {
                    frameIndex = 2;
                } else if (forw.x === curr.x && (forw.y - curr.y === 1 || forw.y - curr.y === -this.size.height + 1) && curr.y === back.y && (curr.x - back.x === -1 || curr.x - back.x === this.size.width - 1)
                    || forw.y === curr.y && (forw.x - curr.x === 1 || forw.x - curr.x === -this.size.width + 1) && curr.x === back.x && (curr.y - back.y === -1 || curr.y - back.y === this.size.height - 1)) {
                    frameIndex = 3;
                } else if (forw.x === curr.x && (forw.y - curr.y === 1 || forw.y - curr.y === -this.size.height + 1) && curr.y === back.y && (curr.x - back.x === 1 || curr.x - back.x === -this.size.width + 1)
                    || forw.y === curr.y && (forw.x - curr.x === -1 || forw.x - curr.x === this.size.width - 1) && curr.x === back.x && (curr.y - back.y === -1 || curr.y - back.y === this.size.height - 1)) {
                    frameIndex = 4;
                }
            }
        }
        this.clearCell(curr);
        this.context.drawImage(player.frames[frameIndex], BORDER_WIDTH + curr.x * this.cellSize, BORDER_WIDTH + curr.y * this.cellSize, this.cellSize, this.cellSize);
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
            case 5:
                this.context.fillStyle = '#9e59ff';
                break;
            case 6:
                this.context.fillStyle = '#e06565';
                break;
            case 7:
                this.context.fillStyle = perk.owner === this.selfId ? '#6b0000' : '#f00000';
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

function HslToRgb(h, s, l) {
    s /= 100;
    l /= 100;
    const k = n => (n + h / 30) % 12;
    const a = s * Math.min(l, 1 - l);
    const f = n =>
        l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)));
    return [Math.round(255 * f(0)), Math.round(255 * f(8)), Math.round(255 * f(4))];
}

function generateFrames(color) {
    const [r, g, b] = HslToRgb(color, 100, 50);
    const frames = [];
    for (let f = 0; f < SPRITE_LENGTH; f++) {
        const imageData = new ImageData(FRAME_SIZE, FRAME_SIZE);
        for (let i = 0; i < FRAME_SIZE * FRAME_SIZE; i++) {
            let pixelIndex = (f * FRAME_SIZE * FRAME_SIZE + i) * 4;
            if (baseSpriteData[pixelIndex + 3] > 0) {
                imageData.data[i * 4] = r / 255 * baseSpriteData[pixelIndex];
                imageData.data[i * 4 + 1] = g / 255 * baseSpriteData[pixelIndex];
                imageData.data[i * 4 + 2] = b / 255 * baseSpriteData[pixelIndex];
                imageData.data[i * 4 + 3] = baseSpriteData[pixelIndex + 3];
            }
        }

        const frame = document.createElement('canvas');
        frame.width = FRAME_SIZE;
        frame.height = FRAME_SIZE;
        frame.getContext('2d').putImageData(imageData, 0, 0);
        frames.push(frame);
    }
    return frames;
}

let baseSpriteData;
const baseSpriteImage = new Image();
baseSpriteImage.addEventListener("load", () => {
    const canvas = document.createElement('canvas');
    canvas.width = FRAME_SIZE;
    canvas.height = SPRITE_LENGTH * FRAME_SIZE;
    const ctx = canvas.getContext('2d');

    ctx.drawImage(baseSpriteImage, 0, 0);
    baseSpriteData = ctx.getImageData(0, 0, FRAME_SIZE, SPRITE_LENGTH * FRAME_SIZE).data;
});
baseSpriteImage.src = 'sprite.png';

new Lobby();
animateTitle();