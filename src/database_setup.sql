-- Drop the database if it exists
DROP DATABASE IF EXISTS chatclient;

CREATE DATABASE IF NOT EXISTS chatclient;
USE chatclient;

# Users
DROP TABLE IF EXISTS users;
CREATE TABLE users (
    Id INT NOT NULL AUTO_INCREMENT,
    UserName VARCHAR(45) NOT NULL,
    Password VARCHAR(255) NOT NULL,
    BlockedUntil INT NOT NULL,
    PRIMARY KEY (Id)
);

INSERT INTO users (UserName, Password, BlockedUntil)
VALUES
('anton', 'dc3e5b292a2a16dc1e28f718b96d3096a1e04c6464a1594edd42824eea771766', 0),
('rino', 'dc3e5b292a2a16dc1e28f718b96d3096a1e04c6464a1594edd42824eea771766', 0),
('testuser', 'dc3e5b292a2a16dc1e28f718b96d3096a1e04c6464a1594edd42824eea771766', 0),
('antonia', 'dc3e5b292a2a16dc1e28f718b96d3096a1e04c6464a1594edd42824eea771766', 0),
('antonius', 'dc3e5b292a2a16dc1e28f718b96d3096a1e04c6464a1594edd42824eea771766', 0);


# Chats
DROP TABLE IF EXISTS chats;
CREATE TABLE chats (
    Id INT NOT NULL AUTO_INCREMENT,
    ChatName VARCHAR(45) NOT NULL,
    User1_Id INT NOT NULL,
    User2_Id INT NOT NULL,
    FOREIGN KEY (User1_Id) REFERENCES users(Id),
    FOREIGN KEY (User2_Id) REFERENCES users(Id),
    PRIMARY KEY (Id)
);

INSERT INTO chats (ChatName, User1_Id, User2_Id)
VALUES ('TestChat1', 1, 2), ('TestChat2', 1, 3);

# Friend Requests
DROP TABLE IF EXISTS friend_requests;
CREATE TABLE friend_requests (
    Id INT NOT NULL AUTO_INCREMENT,
    Sender_Id INT NOT NULL,
    Receiver_Id INT NOT NULL,
    Accepted BOOLEAN NOT NULL,
    FOREIGN KEY (Sender_Id) REFERENCES users(Id),
    FOREIGN KEY (Receiver_Id) REFERENCES users(Id),
    PRIMARY KEY (Id)
);

INSERT INTO friend_requests (Sender_Id, Receiver_Id, Accepted)
VALUES (1, 2, TRUE), (2, 4, FALSE), (1, 3, FALSE), (2, 3, FALSE);

# Friend Mapping TABLE
DROP TABLE IF EXISTS friends;
CREATE TABLE friends (
    Id INT NOT NULL AUTO_INCREMENT,
    User1_Id INT NOT NULL,
    User2_Id INT NOT NULL,
    FOREIGN KEY (User1_Id) REFERENCES users(Id),
    FOREIGN KEY (User2_Id) REFERENCES users(Id),
    PRIMARY KEY (Id)
);

INSERT INTO friends (User1_Id, User2_Id)
VALUES (1, 2);

# Chat Messages
DROP TABLE IF EXISTS chat_messages;
CREATE TABLE chat_messages (
    Id INT NOT NULL AUTO_INCREMENT,
    Chat_Id INT NOT NULL,
    Message TEXT NOT NULL,
    Timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (Chat_Id) REFERENCES chats(Id),
    PRIMARY KEY (Id)
);

INSERT INTO chat_messages (Chat_Id, Message)
VALUES (1, 'rino: Hallo, Anton!'), (1, 'anton: Guten Morgen, Rino!'), # Valid messages for TestChat1
(2, 'Hallo, Testuser!'), (2, 'Morgen, Anton!'); # Invalid Messages for TestChat2


#to refresh changes made to database-strucutre
#docker-compose down -v  # Stoppt die laufenden Container und entfernt Volumes
#docker-compose build    # Baut das Docker-Image basierend auf dem Dockerfile
#docker-compose up -d    # Startet die Container im Hintergrund
#password for all users rn: Ifuckingloverust1



