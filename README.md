# Projeto Dev-Container - Ambiente de Desenvolvimento Docker

Este repositório contém um **ambiente de desenvolvimento completo** utilizando Docker, pronto para PHP 8.0 até o 8.4 (FPM), Apache, MariaDB e Node.js.  

O ambiente foi configurado para ser usado com IDEs como PHPStorm ou VSCode, com **Composer, NPM e Xdebug integrados**.

---

### Estrutura do Projeto

- `docker/` Arquivos de configuração do Docker
- `docker/php/` Dockerfile do PHP-FPM + Composer + Xdebug
- `docker/apache/` Dockerfile do Apache
- `docker/apache/vhosts` Arquivo de configuração por projeto
- `docker/node/` Dockerfile do Node
- `docker/mysql/` Arquivo de configuração
- `.env` Configurações de ambiente (portas, usuários, senhas)
- `docker-compose.yml` Orquestração dos serviços (PHP, Apache, MariaDB)
- `src/` Código-fonte do projeto (montado nos containers)
- `src/my-project` Pasta do projeto
- `src/my-project/public` Pasta pública do projeto (raiz do servidor web, acessada pelo navegador)

```
project-root/
├─ docker/                       
│   ├─ php/                      
│   ├─ node/                      
│   ├─ apache/                   
│   ├─ apache/                   
│   │    └─ vhosts/               
│   └─ mysql/        
├─ docker-compose.yml                          
├─ .env                          
└─ src/                          
    └─ my-project/
        └─ public/    
```
---

### Pré-requisitos

- Docker e Docker Compose instalados
- Sistema operacional compatível (Linux, macOS ou Windows)

---

### Configuração do Ambiente

1. Copie o arquivo `.env.example` para `.env` e ajuste as variáveis conforme necessário:

```dotenv
# Nome base para todos os containers
CONTAINER_NAME=dev_container

# UID/GID para evitar problemas de permissão com volumes
PUID=1000
PGID=1000

# PHP
PHP_PORT=9000
PHP_VERSION=8.4

# Node
NODE_VERSION=22
NODE_PORT=3000
VITE_PORT=5173

# Apache
APACHE_PORT=8000

# MariaDB
MYSQL_ROOT_PASSWORD=password
MYSQL_PORT=3306
```
2. Subir o ambiente com Docker Compose:

```bash
docker compose up -d --build
```
- `--build` garante que as imagens sejam construídas caso haja alterações no Dockerfile.
- O PHP-FPM estará disponível na versão definida em `PHP_VERSION` e na porta definida em `PHP_PORT`.
- O NODE estará disponível na versão definida em `NODE_VERSION` e na porta definida em `NODE_PORT`.
- O Apache estará disponível na porta definida em `APACHE_PORT` (ex: http://localhost:8080).
- O MariaDB estará disponível na porta definida em `MYSQL_PORT`.
- Senha root dos containers PHP e Node é 1234.

---

### Acessando o ambiente

- PHP: integrado ao container php
- Apache: `http://localhost:<APACHE_PORT>`
- MariaDB: host `mysql:<MYSQL_PORT>`, usuário `root` e senha `<MYSQL_ROOT_PASSWORD>` no arquivo `.env`
- Node.js / NPM: dentro do container Node (node -v, npm -v)
- Composer: dentro do container PHP (composer install)

Você pode executar comandos diretamente da IDE apontando para o container PHP `<CONTAINER_NAME>_php` e Node `<CONTAINER_NAME>_node`.

---

### Configuração de Xdebug

- Porta configurada: `9003`
- Host: `host.docker.internal`
- Ativado para debug remoto em IDE
- Exibição de erros do PHP está habilitada `(display_errors=On, error_reporting=E_ALL)`

---

### Volumes e Persistência

- Código-fonte é montado no host `/src` para `/var/www/html` dentro do container
- Banco de dados MariaDB persiste em volume `db_data` para manter dados entre reinicializações

---

### Comandos úteis

Ver logs de containers:
```bash
docker compose logs -f
```

Acessar terminal do container PHP:
```bash
docker exec -it dev_container_php bash
```

Rodar Composer dentro do container PHP:
```bash
docker exec dev_container_php composer install
```

Acessar terminal do container Node:
```bash
docker exec -it dev_container_node bash
```

Rodar NPM dentro do container Node:
```bash
docker exec dev_container_node npm install
```

Parar o ambiente:
```bash
docker compose down
```

---

### Dicas

- Não é necessário instalar PHP, Composer ou Node localmente.
- Para atualizar dependências do Composer: `docker compose exec dev_container_php composer update`
- Para rodar scripts Node/NPM: `docker compose exec dev_container_node npm run <script>`

---

### Instalar Laravel (opcional)


1. Dar permissão para o programa instalador
```bash
chmod +X lara
```
Ao rodar `chmod +x` você está dizendo ao sistema: "Este arquivo agora tem permissão para ser rodado como um programa"

2. Executar o programa instalador
```bash
./lara
```
O comando diz ao sistema: "Execute o arquivo chamado `lara` que está exatamente aqui nesta pasta".

### Atalhos para o Terminal Bash (Opcional)

1. Abra o seu terminal e digite:
```
nano ~/.bashrc
```
2. Adicione o seguinte trecho de código ao seu `~/.bashrc`:
```
# Função única para PHP/Composer no container
docker_php_tools() {
  local tool="$1"
  shift
  local container=$(docker ps --format '{{.Names}}' | grep '_php$' | head -n 1)

  if [ -n "$container" ]; then
    docker exec -it "$container" "$tool" "$@"
  else
    echo "Nenhum container PHP em execução. Rodando '$tool' no host."
    command "$tool" "$@"
  fi
}

# Função única para Node/NPM no container
docker_node_tools() {
  local tool="$1"
  shift
  local container=$(docker ps --format '{{.Names}}' | grep '_node$' | head -n 1)

  if [ -n "$container" ]; then
    docker exec -it "$container" "$tool" "$@"
  else
    echo "Nenhum container Node em execução. Rodando '$tool' no host."
    command "$tool" "$@"
  fi
}

alias up='docker compose up -d'
alias down='docker compose down'
alias php='docker_php_tools php'
alias composer='docker_php_tools composer'
alias npm='docker_node_tools npm'
alias npx='docker_node_tools npx'
alias node='docker_node_tools node'
```
3. Depois, recarregue o Bash:
```bash
source ~/.bashrc
```
Como usar:
```bash
# Ver versão do PHP
php -v

# Instalar dependências do Composer
composer install

# Rodar scripts NPM
npm run dev

# Ver versão do Node
node -v
```
- O comando detecta automaticamente o container do projeto que utiliza este repositório.
- Caso o container não esteja rodando, o comando será executado no host.

Feito para simplificar o desenvolvimento em projetos PHP modernos, integrando debug, Composer, Node e banco de dados em containers separados, mas trabalhando de forma integrada com a IDE.
