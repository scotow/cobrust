/* global ByteBuffer, FRAMES_MAPPING */

const SPRITE_LENGTH = 16;
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
                const multiSnake = document.getElementById('create-multi-snake').checked ? 1 : 0;
                const perkSpacing = document.getElementById('create-perk-spacing-group').classList.contains('hidden') ? 1 : Number(document.getElementById('create-perk-spacing').value);

                const nameData = new ByteBuffer(0, ByteBuffer.BIG_ENDIAN, true);
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
                data.writeUnsignedByte(multiSnake);
                data.writeUnsignedShort(perkSpacing);
                this.socket.send(data.buffer);
            });
        });
    }

    setupEvents() {
        function updateForm() {
            document.getElementById('create-perk-spacing-group').classList.toggle('hidden', Array.from(document.querySelectorAll('input[type=checkbox].perk')).every((perk) => !perk.checked));
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
        default:
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
            this.games[String(id)] = new LobbyGame(id, {
                name, size, speed, playerCount,
            });
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

        this.game.append(
            name,
            size,
            separator.cloneNode(),
            speed,
            separator.cloneNode(),
            this.players,
            separator.cloneNode(),
            join,
        );
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
            this.changePlayerColor(data);
            break;
        case 6:
            this.snakeChanges(data);
            break;
        default:
            break;
        }
    }

    processSwipe(x, y) {
        this.socket.send(
            new Uint8Array([0, Math.abs(x) > Math.abs(y) ? (x < 0 ? 2 : 3) : (y < 0 ? 0 : 1)]),
        );
    }

    processKey(event) {
        let data;
        switch (event.code) {
        case 'ArrowUp':
        case 'KeyW':
            data = [0, 0];
            break;
        case 'ArrowDown':
        case 'KeyS':
            data = [0, 1];
            break;
        case 'ArrowLeft':
        case 'KeyA':
            data = [0, 2];
            break;
        case 'ArrowRight':
        case 'KeyD':
            data = [0, 3];
            break;
        case 'KeyC':
            data = [1];
            break;
        default:
            return;
        }
        this.socket.send(new Uint8Array(data));
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
            if (typeof additionalHeight !== 'number') {
                additionalHeight = 0;
            }

            const mainSize = document.getElementById('main').getBoundingClientRect();
            this.cellSize = Math.max(
                Math.floor(mainSize.width / this.size.width),
                Math.floor((mainSize.height + additionalHeight) / this.size.height),
            );
            this.canvas.width = this.size.width * this.cellSize + 2 * BORDER_WIDTH;
            this.canvas.height = this.size.height * this.cellSize + 2 * BORDER_WIDTH;

            const scale = Math.min(
                (mainSize.width - 60) / this.canvas.width,
                (mainSize.height + additionalHeight - 27 - 60) / this.canvas.height,
            );
            this.canvas.style.width = `${Math.floor(this.canvas.width * scale)}px`;
            this.canvas.style.height = `${Math.floor(this.canvas.height * scale)}px`;

            for (const player of Object.values(this.players)) {
                player.frames = this.generateFrames(player.color);
            }
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

        const actions = document.createElement('div');
        actions.classList.add('actions');

        this.changeColor = document.createElement('div');
        this.changeColor.classList.add('action', 'change-color', 'hidden');
        this.changeColor.title = 'Change color';
        this.changeColor.addEventListener('click', () => {
            this.socket.send(new Uint8Array([1]));
        });

        const leave = document.createElement('div');
        leave.classList.add('action', 'leave');
        leave.innerText = 'Leave';
        leave.title = 'Back to lobby';
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

        actions.append(this.changeColor, leave);
        header.append(title, actions);
        document.getElementById('game').append(header, this.canvas);
        document.body.classList.replace('lobbying', 'playing');
    }

    redrawCanvas() {
        this.drawBorders();
        this.emptyCanvas();

        for (const player of Object.values(this.players)) {
            for (const body of Object.values(player.bodies)) {
                for (let i = 0; i < body.length; i += 1) {
                    this.drawFrame(body, i, player.frames);
                }
            }
        }
        for (const perk of Object.values(this.perks)) {
            this.drawPerk(perk);
        }
    }

    drawBorders() {
        this.context.strokeStyle = '#ffffff';
        this.context.lineWidth = BORDER_WIDTH;
        this.context.strokeRect(
            BORDER_WIDTH,
            BORDER_WIDTH,
            this.canvas.width - 2 * BORDER_WIDTH,
            this.canvas.height - 2 * BORDER_WIDTH,
        );
    }

    emptyCanvas() {
        this.context.clearRect(
            BORDER_WIDTH,
            BORDER_WIDTH,
            this.canvas.width - 2 * BORDER_WIDTH,
            this.canvas.height - 2 * BORDER_WIDTH,
        );
    }

    setPlayers(data) {
        while (data.available) {
            const playerId = data.readUnsignedShort();
            const color = data.readUnsignedShort();
            const frames = this.generateFrames(color);
            const nbBody = data.readUnsignedByte();
            const bodies = {};
            for (let b = 0; b < nbBody; b += 1) {
                const body = [];
                const bodyId = data.readUnsignedShort();
                const size = data.readUnsignedShort();
                for (let i = 0; i < size; i += 1) {
                    const cell = {
                        x: data.readUnsignedShort(),
                        y: data.readUnsignedShort(),
                    };
                    body.push(cell);
                }
                bodies[bodyId] = body;

                for (let i = 0; i < size; i += 1) {
                    this.drawFrame(body, i, frames);
                }
            }
            this.players[playerId] = { color, bodies, frames: this.generateFrames(color) };

            if (playerId === this.selfId) {
                this.updateChangeColorButton(color);
            }
        }
    }

    addPlayer(data) {
        const playerId = data.readUnsignedShort();
        const bodyId = data.readUnsignedShort();
        const color = data.readUnsignedShort();
        const bodies = {};
        bodies[bodyId] = [{
            x: data.readUnsignedShort(),
            y: data.readUnsignedShort(),
        }];
        this.players[playerId] = { color, bodies, frames: this.generateFrames(color) };
        this.drawFrame(bodies[bodyId], 0, this.players[playerId].frames);
    }

    removePlayer(data) {
        const playerId = data.readUnsignedShort();
        for (const body of Object.values(this.players[playerId].bodies)) {
            this.clearCell(body);
        }
        delete this.players[playerId];
    }

    changePlayerColor(data) {
        const playerId = data.readUnsignedShort();
        const color = data.readUnsignedShort();
        const player = this.players[playerId];
        if (player === undefined) {
            return;
        }
        player.color = color;
        player.frames = this.generateFrames(color);
        for (const body of Object.values(player.bodies)) {
            for (let i = 0; i < body.length; i += 1) {
                this.drawFrame(body, i, player.frames);
            }
        }
        if (playerId === this.selfId) {
            this.updateChangeColorButton(color);
        }
    }

    updateChangeColorButton(color) {
        this.changeColor.style.backgroundColor = `hsl(${color}, 100%, 35%)`;
        this.changeColor.classList.remove('hidden');
    }

    snakeChanges(data) {
        while (data.available) {
            switch (data.readUnsignedByte()) {
            case 0: {
                const player = this.players[data.readUnsignedShort()];
                const body = player.bodies[data.readUnsignedShort()];
                this.clearCell(body.pop());
                this.drawFrame(body, body.length - 1, player.frames);
            } break;
            case 1: {
                const player = this.players[data.readUnsignedShort()];
                const body = player.bodies[data.readUnsignedShort()];
                const head = {
                    x: data.readUnsignedShort(),
                    y: data.readUnsignedShort(),
                };
                body.unshift(head);
                delete this.perks[`${head.x},${head.y}`];

                if (body.length >= 2) {
                    this.drawFrame(body, 1, player.frames);
                }
                this.drawFrame(body, 0, player.frames);
            } break;
            case 2: {
                const player = this.players[data.readUnsignedShort()];
                const bodyId = data.readUnsignedShort();
                const head = {
                    x: data.readUnsignedShort(),
                    y: data.readUnsignedShort(),
                };
                player.bodies[bodyId] = [head];
                this.drawFrame(player.bodies[bodyId], 0, player.frames);
            } break;
            case 3: {
                const player = this.players[data.readUnsignedShort()];
                const bodyId = data.readUnsignedShort();
                this.clearCell(player.bodies[bodyId]);
                delete player.bodies[bodyId];
            } break;
            case 4: {
                const player = this.players[data.readUnsignedShort()];
                for (const bodyId of Object.keys(player.bodies)) {
                    const body = player.bodies[bodyId];
                    player.bodies[bodyId] = body.reverse();
                    this.drawFrame(body, 0, player.frames);
                    this.drawFrame(body, body.length - 1, player.frames);
                }
            } break;
            default:
                break;
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
            default:
                break;
            }
            this.perks[`${coord.x},${coord.y}`] = perk;
            this.drawPerk(perk);
        }
    }

    clearCell(coords) {
        for (const { x, y } of coords instanceof Array ? coords : [coords]) {
            this.context.clearRect(
                BORDER_WIDTH + x * this.cellSize,
                BORDER_WIDTH + y * this.cellSize,
                this.cellSize,
                this.cellSize,
            );
        }
    }

    drawFrame(body, index, frames) {
        const cellIndexToFrameIndex = () => {
            if (index === 0 && body.length === 1) {
                return 0; // Egg.
            }

            const forw = body[index - 1] ?? null;
            const curr = body[index] ?? null;
            const back = body[index + 1] ?? null;

            let state = 0;
            for (const [lhs, rhs] of [[curr, back], [forw, curr]]) {
                state <<= 5;
                if (lhs === null || rhs === null) {
                    continue;
                }

                let dx = lhs.x.absDiff(rhs.x);
                if (dx === this.size.width - 1) dx = 1;
                let dy = lhs.y.absDiff(rhs.y);
                if (dy === this.size.height - 1) dy = 1;

                if (dx <= 1 && dy <= 1 && dx ^ dy === 1) {
                    state |= 1 << 0;
                    state |= (lhs.x - rhs.x === -1 || lhs.x - rhs.x === this.size.width - 1) << 1;
                    state |= (lhs.x - rhs.x === 1 || lhs.x - rhs.x === -this.size.width + 1) << 2;
                    state |= (lhs.y - rhs.y === -1 || lhs.y - rhs.y === this.size.height - 1) << 3;
                    state |= (lhs.y - rhs.y === 1 || lhs.y - rhs.y === -this.size.height + 1) << 4;
                }
            }

            return FRAMES_MAPPING.get(state << 1 | index === 0);
        };

        this.clearCell(body[index]);
        this.context.putImageData(
            frames[cellIndexToFrameIndex()],
            BORDER_WIDTH + body[index].x * this.cellSize,
            BORDER_WIDTH + body[index].y * this.cellSize,
        );
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
        case 8:
            this.context.fillStyle = '#00ff4c';
            break;
        default: return;
        }
        this.context.beginPath();
        this.context.arc(
            BORDER_WIDTH + perk.coord.x * this.cellSize + this.cellSize / 2,
            BORDER_WIDTH + perk.coord.y * this.cellSize + this.cellSize / 2,
            this.cellSize / 4,
            0,
            2 * Math.PI,
        );
        this.context.fill();
    }

    generateFrames(color) {
        const [r, g, b] = hslToRgb(color, 100, 50);
        const frames = [];

        const canvas = document.createElement('canvas');
        canvas.width = this.cellSize;
        canvas.height = SPRITE_LENGTH * this.cellSize;
        const ctx = canvas.getContext('2d');
        ctx.drawImage(baseSpriteImage, 0, 0, this.cellSize, SPRITE_LENGTH * this.cellSize);
        const templateData = ctx.getImageData(
            0,
            0,
            this.cellSize,
            SPRITE_LENGTH * this.cellSize,
        ).data;

        for (let f = 0; f < SPRITE_LENGTH; f += 1) {
            const imageData = new ImageData(this.cellSize, this.cellSize);
            for (let i = 0; i < this.cellSize * this.cellSize; i += 1) {
                const pixelIndex = (f * this.cellSize * this.cellSize + i) * 4;
                if (templateData[pixelIndex + 3] > 0) {
                    imageData.data[i * 4] = r / 255 * templateData[pixelIndex];
                    imageData.data[i * 4 + 1] = g / 255 * templateData[pixelIndex];
                    imageData.data[i * 4 + 2] = b / 255 * templateData[pixelIndex];
                    imageData.data[i * 4 + 3] = templateData[pixelIndex + 3];
                }
            }
            frames.push(imageData);
        }
        return frames;
    }
}

function baseWebsocketUrl() {
    return `${window.location.protocol.slice(0, -1) === 'https' ? 'wss' : 'ws'}://${window.location.host}`;
}

(function animateTitle() {
    const context = document.getElementById('title').getContext('2d');
    function fillCell(color, { x, y }, shift) {
        context.fillStyle = color;
        context.fillRect(x * 25 + 1, y * 25 + 1 + (shift ? 12 : 0), 23, 23);
    }

    /* eslint-disable max-len */
    const letters = [
        {
            color: 'red',
            frames: [[{ x: 2, y: 0 }], [{ x: 1, y: 0 }], [{ x: 0, y: 0 }], [{ x: 0, y: 1 }], [{ x: 0, y: 2 }], [{ x: 1, y: 2 }], [{ x: 2, y: 2 }], [{ x: 2, y: 3 }], [{ x: 2, y: 4 }], [{ x: 1, y: 4 }], [{ x: 0, y: 4 }]],
        },
        {
            color: 'green',
            frames: [[{ x: 4, y: 4 }], [{ x: 4, y: 3 }], [{ x: 4, y: 2 }], [{ x: 4, y: 1 }], [{ x: 4, y: 0 }], [{ x: 5, y: 0 }], [{ x: 6, y: 0 }], [{ x: 6, y: 0 }], [{ x: 6, y: 1 }], [{ x: 6, y: 2 }], [{ x: 6, y: 3 }], [{ x: 6, y: 4 }]],
        },
        {
            color: 'purple',
            frames: [[{ x: 8, y: 4 }], [{ x: 8, y: 3 }], [{ x: 8, y: 2 }], [{ x: 8, y: 1 }], [{ x: 8, y: 0 }], [{ x: 9, y: 0 }], [{ x: 10, y: 0 }], [{ x: 10, y: 0 }], [{ x: 10, y: 1 }], [{ x: 10, y: 2 }, { x: 9, y: 2 }], [{ x: 10, y: 3 }], [{ x: 10, y: 4 }]],
        },
        {
            color: 'blue',
            frames: [[{ x: 12, y: 0 }], [{ x: 12, y: 1 }], [{ x: 12, y: 2 }], [{ x: 12, y: 3 }, { x: 13, y: 2 }], [{ x: 12, y: 4 }, { x: 14, y: 1 }, { x: 14, y: 3 }], [{ x: 14, y: 0 }, { x: 14, y: 4 }]],
        },
        {
            color: 'orange',
            frames: [[{ x: 18, y: 0 }], [{ x: 17, y: 0 }], [{ x: 16, y: 0 }], [{ x: 16, y: 1 }], [{ x: 16, y: 2 }], [{ x: 16, y: 3 }, { x: 17, y: 2 }], [{ x: 16, y: 4 }], [{ x: 17, y: 4 }], [{ x: 18, y: 4 }]],
        },
    ];
    /* eslint-enable max-len */

    for (let l = 0; l < letters.length; l += 1) {
        for (let i = 0; i < letters[l].frames.length; i += 1) {
            setTimeout(() => {
                for (const cell of letters[l].frames[i]) {
                    fillCell(letters[l].color, cell, l % 2 === 1);
                }
            }, i * 100);
        }
    }
}());

Number.prototype.absDiff = function (other) {
    return other > this ? other - this : this - other;
};

function hslToRgb(h, s, l) {
    s /= 100;
    l /= 100;
    const k = (n) => (n + h / 30) % 12;
    const a = s * Math.min(l, 1 - l);
    const f = (n) => l - a * Math.max(-1, Math.min(k(n) - 3, Math.min(9 - k(n), 1)));
    return [Math.round(255 * f(0)), Math.round(255 * f(8)), Math.round(255 * f(4))];
}

const baseSpriteImage = new Image();
baseSpriteImage.addEventListener('load', () => {
    new Lobby();
});
baseSpriteImage.src = 'sprite.png';
