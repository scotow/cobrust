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
            if (!nameInput.value.trim()) {
                nameInput.value = `Game ${1000 + Math.floor(Math.random() * 8999)}`;
                if (shouldAutoKeyboard()) {
                    nameInput.select();
                    nameInput.focus();
                }
            } else if (shouldAutoKeyboard()) {
                nameInput.focus();
            }
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
        let frameIndex;
        if (index === 0 && body.length === 1) {
            frameIndex = 0;
        } else {
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

            frameIndex = FRAMES_MAPPING.get(state << 1 | index === 0);
        }

        this.clearCell(body[index]);
        this.context.putImageData(
            frames[frameIndex],
            BORDER_WIDTH + body[index].x * this.cellSize,
            BORDER_WIDTH + body[index].y * this.cellSize,
        );
    }

    drawPerk(perk) {
        let color;
        let icon;
        let iconWidth = 28;
        const iconHeight = 28;
        switch (perk.id) {
        case 0: // Food
            color = '#2fbf71';
            icon = 'M1 15.8727C1 17.4144 2.05445 18.4476 3.60577 18.4476H8.69171V23.5431C8.69171 25.0965 9.72272 26.1372 11.2644 26.1372H15.8844C17.4261 26.1372 18.4593 25.0965 18.4593 23.5431V18.4476H23.5548C25.1083 18.4476 26.1489 17.4144 26.1489 15.8727V11.2527C26.1489 9.71101 25.1083 8.67789 23.5548 8.67789H18.4593V3.59405C18.4593 2.04273 17.4261 1 15.8844 1H11.2644C9.72272 1 8.69171 2.04273 8.69171 3.59405V8.67789H3.59405C2.04273 8.67789 1 9.71101 1 11.2527V15.8727Z';
            break;
        case 1: // Reserved food
            color = perk.owner === this.selfId ? '#1e90ff' : '#0c3b66';
            icon = 'M1 15.8727C1 17.4144 2.05445 18.4476 3.60577 18.4476H8.69171V23.5431C8.69171 25.0965 9.72272 26.1372 11.2644 26.1372H15.8844C17.4261 26.1372 18.4593 25.0965 18.4593 23.5431V18.4476H23.5548C25.1083 18.4476 26.1489 17.4144 26.1489 15.8727V11.2527C26.1489 9.71101 25.1083 8.67789 23.5548 8.67789H18.4593V3.59405C18.4593 2.04273 17.4261 1 15.8844 1H11.2644C9.72272 1 8.69171 2.04273 8.69171 3.59405V8.67789H3.59405C2.04273 8.67789 1 9.71101 1 11.2527V15.8727Z';
            break;
        case 2: // Reverser
            color = '#f0c808';
            icon = 'm 4.1288302,18.685972 3.2339102,0.05453 c 0.00893,-3.263923 -0.018019,-6.528711 0.013748,-9.7920841 0.022236,-1.4969842 0.7517808,-2.5403667 2.3060583,-2.4607117 1.4880783,-0.0755 2.0451303,0.8320148 2.1339083,2.2833631 0.07075,2.7647127 0.01231,5.5343957 0.0314,8.3010927 -0.117338,2.210136 0.961704,4.881224 3.081934,5.803313 2.280734,1.047148 6.506333,0.622671 7.761416,-1.644227 1.120829,-1.832615 1.056196,-4.658256 1.110799,-6.699168 L 23.720212,7.8333584 20.404541,7.91515 c -0.0089,3.316206 -0.0093,6.524222 -0.04101,9.839875 -0.02399,1.494999 -1.055315,2.750012 -2.605941,2.671236 -1.555338,0.08146 -2.562984,-1.022273 -2.587511,-2.531752 -0.04803,-3.006075 -0.02734,-6.042567 -0.09405,-9.0481059 C 14.969881,6.6547394 13.535749,4.3036246 11.338498,3.752133 9.134779,3.110799 6.2066337,4.062546 5.1190837,6.160695 4.0636463,7.9966742 4.1888433,10.187317 4.1288302,12.207191 Z M 2.89458,17.9553 c -1.1197669,-0.09455 -0.9977982,1.209058 -0.3684717,1.745161 0.9094483,1.248434 1.7704474,2.535968 2.7109338,3.759363 0.702472,0.679546 1.3780925,-0.175223 1.7275401,-0.762617 C 7.8075716,21.482865 8.7044631,20.302556 9.5120813,19.065868 9.9927323,18.129532 8.8640972,17.835116 8.1651525,17.9553 Z M 19.0981,8.76007 C 21.142066,8.740859 23.190923,8.798959 25.23176,8.730179 26.235439,8.48944 25.686527,7.4160005 25.223942,6.9110265 24.342785,5.6950712 23.512966,4.4377255 22.598882,3.2483055 21.898315,2.576891 21.21676,3.4172183 20.8679,4.0056897 20.02227,5.2192892 19.122193,6.3983046 18.312328,7.6346338 17.954682,8.2369132 18.457248,8.8253737 19.0981,8.76007 Z';
            break;
        case 3: // Teleporter
            color = '#e7820e';
            icon = 'M6.96679 18.0529H7.81687C7.72336 17.3233 7.66898 16.5435 7.78476 15.8603H7.0957C4.29515 15.8603 2.31608 13.8962 2.31608 11.0744C2.31608 8.27382 4.30897 6.29803 7.0957 6.29803H17.4586C20.2474 6.29803 22.2382 8.27382 22.2382 11.0744C22.2382 13.8962 20.2495 15.8603 17.4586 15.8603H13.6912C13.369 16.3427 13.4191 17.478 13.909 18.0529H17.5875C21.6551 18.0529 24.5543 15.19 24.5543 11.0744C24.5543 6.96836 21.6551 4.10547 17.5875 4.10547H6.96679C2.89922 4.10547 0 6.96836 0 11.0744C0 15.19 2.89922 18.0529 6.96679 18.0529ZM16.4053 23.5886H27.026C31.0936 23.5886 33.9928 20.7257 33.9928 16.6101C33.9928 12.5041 31.0936 9.64117 27.026 9.64117H26.1663C26.2619 10.359 26.3142 11.1484 26.208 11.8337H26.8875C29.688 11.8337 31.6788 13.7978 31.6788 16.6101C31.6788 19.4202 29.6859 21.396 26.8875 21.396H16.5363C13.7475 21.396 11.7546 19.4202 11.7546 16.6101C11.7546 13.7978 13.7337 11.8337 16.5363 11.8337H20.3016C20.6006 11.3282 20.5505 10.2161 20.0763 9.64117H16.4053C12.3281 9.64117 9.43851 12.5041 9.43851 16.6101C9.43851 20.7257 12.3281 23.5886 16.4053 23.5886Z';
            iconWidth = 34;
            break;
        case 4: // Speed boost
            color = '#e70ed9';
            icon = 'M5 15.8367C5 16.3254 5.37687 16.6842 5.89648 16.6842H12.6863L9.1182 26.2861C8.63375 27.5627 9.95937 28.2431 10.7973 27.206L21.7428 13.6615C21.9547 13.4027 22.0622 13.1557 22.0622 12.8735C22.0622 12.3923 21.6854 12.0239 21.1658 12.0239H14.3759L17.944 2.42203C18.4306 1.1475 17.1029 0.467114 16.265 1.51172L5.31945 15.0487C5.10758 15.315 5 15.5641 5 15.8367Z';
            break;
        case 5: // Food frenzy
            color = '#9e59ff';
            icon = 'M9.57164 20.1838H23.2137C23.7614 20.1838 24.2463 19.7582 24.2463 19.1488C24.2463 18.5416 23.7614 18.1202 23.2137 18.1202H9.83203C9.26109 18.1202 8.90999 17.7147 8.82139 17.1053L6.94381 4.22687C6.80975 3.21906 6.39937 2.70929 5.1007 2.70929H1.08233C0.499683 2.70929 0 3.21437 0 3.80335C0 4.39983 0.499683 4.90701 1.08233 4.90701H4.77539L6.59133 17.3439C6.85008 19.1008 7.78805 20.1838 9.57164 20.1838ZM7.52085 15.8917H23.3004C25.0894 15.8917 26.0316 14.8033 26.2882 13.0305L27.173 7.1371C27.1964 6.97843 27.2241 6.78249 27.2241 6.65171C27.2241 5.98375 26.7494 5.51477 25.9509 5.51477H6.47179L6.4814 7.59014H24.8206L24.0687 12.8195C23.9876 13.4406 23.6599 13.8259 23.0698 13.8259H7.50163L7.52085 15.8917ZM10.527 26.0415C11.6555 26.0415 12.5562 25.1483 12.5562 24.0102C12.5562 22.8859 11.6555 21.981 10.527 21.981C9.39305 21.981 8.48602 22.8859 8.48602 24.0102C8.48602 25.1483 9.39305 26.0415 10.527 26.0415ZM21.3638 26.0415C22.4998 26.0415 23.4026 25.1483 23.4026 24.0102C23.4026 22.8859 22.4998 21.981 21.3638 21.981C20.2395 21.981 19.3249 22.8859 19.3249 24.0102C19.3249 25.1483 20.2395 26.0415 21.3638 26.0415Z';
            break;
        case 6: // Mines trail
            color = '#e06565';
            icon = 'M13.3923 25.6767C19.698 25.6767 23.9031 21.4235 23.9031 15.0239C23.9031 4.42804 14.8448 0 8.59913 0C7.39843 0 6.61516 0.449997 6.61516 1.30124C6.61516 1.63218 6.76094 1.97905 7.02601 2.28561C8.47328 4.0235 9.80875 5.9196 9.82797 8.17335C9.82797 8.62265 9.77992 9.01969 9.4682 9.5768L10.0098 9.45539C9.38969 7.63782 8.06453 6.5625 6.8814 6.5625C6.33133 6.5625 5.9568 6.94992 5.9568 7.55742C5.9568 7.91906 6.04094 8.56851 6.04094 9.14109C6.04094 11.823 4 13.2105 4 17.5317C4 22.4217 7.7439 25.6767 13.3923 25.6767ZM13.5159 23.707C9.00436 23.707 5.98139 21.222 5.98139 17.5317C5.98139 13.8335 7.99022 12.6466 7.98601 9.32343C7.98601 8.90648 7.90069 8.52796 7.81421 8.19749L7.44765 8.57765C8.35164 9.26155 8.9657 10.3931 9.30906 11.9269C9.36437 12.2383 9.54718 12.4062 9.79679 12.4062C10.8662 12.4062 11.5881 9.97475 11.5881 8.29242C11.5881 5.64984 10.4617 3.09726 8.62772 1.36382L8.19108 1.87171C16.4713 2.10655 21.8493 7.57616 21.8493 14.9611C21.8493 20.1907 18.5099 23.707 13.5159 23.707ZM13.6787 22.0695C16.3855 22.0695 17.7723 20.103 17.7723 17.7848C17.7723 15.4617 16.4301 12.9703 13.9164 11.8139C13.7751 11.764 13.6605 11.8437 13.6848 11.9955C13.8946 13.8119 13.6649 15.492 13.0649 16.3905C12.7888 15.6919 12.4478 15.1144 11.9341 14.6444C11.8162 14.5404 11.7016 14.603 11.6802 14.738C11.4991 16.1297 10.0743 16.811 10.0743 18.8292C10.0743 20.7684 11.5105 22.0695 13.6787 22.0695Z';
            break;
        case 7: // Mine
            color = perk.owner === this.selfId ? '#6b0000' : '#f00000';
            icon = 'M13.3923 25.6767C19.698 25.6767 23.9031 21.4235 23.9031 15.0239C23.9031 4.42804 14.8448 0 8.59913 0C7.39843 0 6.61516 0.449997 6.61516 1.30124C6.61516 1.63218 6.76094 1.97905 7.02601 2.28561C8.47328 4.0235 9.80875 5.9196 9.82797 8.17335C9.82797 8.62265 9.77992 9.01969 9.4682 9.5768L10.0098 9.45539C9.38969 7.63782 8.06453 6.5625 6.8814 6.5625C6.33133 6.5625 5.9568 6.94992 5.9568 7.55742C5.9568 7.91906 6.04094 8.56851 6.04094 9.14109C6.04094 11.823 4 13.2105 4 17.5317C4 22.4217 7.7439 25.6767 13.3923 25.6767ZM13.5159 23.707C9.00436 23.707 5.98139 21.222 5.98139 17.5317C5.98139 13.8335 7.99022 12.6466 7.98601 9.32343C7.98601 8.90648 7.90069 8.52796 7.81421 8.19749L7.44765 8.57765C8.35164 9.26155 8.9657 10.3931 9.30906 11.9269C9.36437 12.2383 9.54718 12.4062 9.79679 12.4062C10.8662 12.4062 11.5881 9.97475 11.5881 8.29242C11.5881 5.64984 10.4617 3.09726 8.62772 1.36382L8.19108 1.87171C16.4713 2.10655 21.8493 7.57616 21.8493 14.9611C21.8493 20.1907 18.5099 23.707 13.5159 23.707ZM13.6787 22.0695C16.3855 22.0695 17.7723 20.103 17.7723 17.7848C17.7723 15.4617 16.4301 12.9703 13.9164 11.8139C13.7751 11.764 13.6605 11.8437 13.6848 11.9955C13.8946 13.8119 13.6649 15.492 13.0649 16.3905C12.7888 15.6919 12.4478 15.1144 11.9341 14.6444C11.8162 14.5404 11.7016 14.603 11.6802 14.738C11.4991 16.1297 10.0743 16.811 10.0743 18.8292C10.0743 20.7684 11.5105 22.0695 13.6787 22.0695Z';
            break;
        case 8: // Multi-snake
            color = '#5eeaf7';
            icon = 'M14.1601 25.1634C14.9092 25.1634 15.321 24.6856 15.321 23.801V17.239C15.321 14.4708 18.4571 10.5319 21.1501 8.81648L22.0961 8.205C22.4643 7.98047 22.6675 7.57711 22.6675 7.18125C22.6675 6.53532 22.2332 6.06517 21.5322 6.06517C21.2075 6.06517 20.8616 6.17696 20.5508 6.38134L19.934 6.78728C17.331 8.51908 14.63 11.8953 14.1761 13.7644H14.1367C13.6806 11.8856 10.9893 8.51908 8.38628 6.78728L7.76941 6.38134C7.45113 6.17485 7.11269 6.06306 6.78058 6.06306C6.07535 6.06306 5.65488 6.56837 5.65488 7.17165C5.65488 7.5675 5.85598 7.97625 6.22418 8.20289L7.17012 8.81648C9.84926 10.5319 13.0014 14.4708 13.0014 17.239V23.801C13.0014 24.6856 13.4132 25.1634 14.1601 25.1634ZM5.92887 10.2504L9.50121 5.63369C10.0431 4.93103 9.67394 4.32564 8.78965 4.2926L3.9695 4.09971C3.24364 4.06666 2.84357 4.57197 3.05755 5.27767L4.4495 9.88477C4.70286 10.7468 5.37528 10.9648 5.92887 10.2504ZM22.2318 10.2323C22.7598 10.9648 23.446 10.7723 23.7174 9.92203L25.2531 5.3522C25.4809 4.65822 25.0946 4.13908 24.3784 4.14658L19.5486 4.1958C18.6643 4.2033 18.2814 4.79275 18.7977 5.50923L22.2318 10.2323Z';
            break;
        default: return;
        }
        this.context.fillStyle = color;
        this.context.beginPath();
        this.context.arc(
            BORDER_WIDTH + perk.coord.x * this.cellSize + this.cellSize / 2,
            BORDER_WIDTH + perk.coord.y * this.cellSize + this.cellSize / 2,
            this.cellSize / 3,
            0,
            2 * Math.PI,
        );
        this.context.fill();

        if (this.cellSize >= 15) {
            this.context.fillStyle = '#ffffff';
            this.context.setTransform(
                this.cellSize / 2 / iconWidth,
                0,
                0,
                this.cellSize / 2 / iconHeight,
                BORDER_WIDTH + perk.coord.x * this.cellSize + this.cellSize / 4,
                BORDER_WIDTH + perk.coord.y * this.cellSize + this.cellSize / 4,
            );
            this.context.fill(new Path2D(icon));
            this.context.setTransform(1, 0, 0, 1, 0, 0);
        }
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

function shouldAutoKeyboard() {
    return !/Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent);
}

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
