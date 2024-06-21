
### Entwicklung einer sicheren Chatanwendung - Gruppe 03

##### Autoren:
- Schuster, Rinaldo
- Hülß, Anton

### Ziel des Projekts
- Ziel ist es, im Rahmen des Moduls "Secure Software Engineering" eine sichere Chat-Anwendung in Rust zu implementieren
- Nachrichten müssen dabei vor Zugriffen von Außenstehenden vertraulich behandelt werden 
- Außerdem müssen sich Benutzer dem System gegenüber authentifizieren 
- Die Anwendung soll es mehreren Benutzern erlauben, miteinander kommunizieren zu können
- Nachrichten müssen während der Übertragung auf ihre Integrität geprüft werden.

### Anforderungen
- Sämtliche Anforderungen können auf RedMine gefunden werden. Einzelne Anforderungen befinden sich zusätzlich noch einmal in den anderen Markdown-Dateien. 

### Architektur
- Client-Server-Architektur
- MySQL-Datenbank zur Speicherung der Nutzerprofile, Chatverläufe und weitere Daten
- Die Datenbank läuft auf dem Server der Hochschule Coburg, weshalb eine durchgehende Verbindung zum Netzwerk der Hochschule benötigt wird (entweder vor Ort oder mittels VPN)

##### Warum MySQL?
- MySQL bietet hohe Leistung, Skalierbarkeit und Flexibilität, unterstützt durch eine starke Open-Source-Community
- Mit robusten Sicherheitsfunktionen wie Benutzerverwaltung und Datenverschlüsselung schützt MySQL sensible Daten zuverlässig
- Zudem ist MySQL plattformunabhängig und unterstützt verschiedene Betriebssysteme
- Insgesamt ist MySQL eine zuverlässige und effiziente Wahl für Datenbanklösungen

##### Warum BCrypt?
- BCrypt bietet eine sichere und bewährte Methode zum Passwort-Hashing 
- Es verwendet adaptive Hashing, das gegen Brute-Force-Angriffe resistent ist, und kann an die steigende Rechenleistung angepasst werden, indem die Rechenkomplexität erhöht wird 
- Zudem gibt es gut unterstützte Rust-Bibliotheken für BCrypt, die eine einfache Integration in unser Projekt ermöglichen
- BCrypt verwendet Salted Hashing, wodurch selbst identische Passwörter unterschiedliche Hashes erhalten und so vor Rainbow-Table-Angriffen geschützt sind
- Seine Resistenz gegen GPU-basierte Angriffe erhöht die Sicherheit weiter 
- BCrypt ermöglicht zudem einfache Anpassungen der Work-Factor, um die Hash-Funktion im Laufe der Zeit sicher zu halten 
- Die erprobte Sicherheit und umfassende Prüfung machen BCrypt zu einer vertrauenswürdigen Wahl für das Passwort-Hashing
- Insgesamt verbessert BCrypt die Sicherheit von Anwendungen, indem es robuste Schutzmechanismen für gespeicherte Passwörter bietet

### Sicherheitsbedenken
- Ein Teil der Sicherheitsbedenken und getroffene Maßnahmen sind auf RedMine dokumentiert. Im Folgenden werden weitere Sicherheitsbedenken dokumentiert.
- Die Verschlüsselung der Kommunikation mittels TLS funktioniert noch nicht so wie gewünscht
    - Ziel ist es, bis zum zweiten Release dies zu beheben oder gar eine Ende-zu-Ende-Verschlüsselung zu implementieren

### Testen und Coverage
- Aufgrund unserer Architektur ist es schwierig und nicht hunderprozentig zielführend, alle Funktionen mit Unit-Tests zu testen
- Besonders interaktive Funktionen, bei denen auf Nutzereingaben gewartet wird, müssten unnötig komplex umstrukturiert werden, ohne einen nennenswerten Nutzen zu bringen
- Wichtige Kernfunktionen, wie die Interaktion mit der Datenbank, wurden jedoch ausreichend getestet
- Dennoch setzen wir uns das Ziel, die Test-Coverage auch für das zweite Release an einigen Stellen noch zu erhöhen

### Zukünftige Entwicklung
- Gruppenchats - mehr als zwei Personen sollen gleichzeitig in einem Chat kommunizieren können
- Verstärkte Authentifizierung - beispielsweise durch eine Zwei-Faktor-Authentifizierung über die E-Mail Adresse eines Nutzers
- TLS-Verschlüsselung für die Datenübertragung
- Verschlüsselung der Nachrichten in der Datenbank


