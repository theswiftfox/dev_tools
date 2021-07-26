// Get the modal
var modal = null;

// When the user clicks anywhere outside of the modal, close it
window.onclick = function (event) {
    if (modal == null) {
        modal = document.getElementById('login_form');
    }
    if (event.target == modal) {
        modal.style.display = 'none';
    }
}

function showLoginForm(show) {
    let loginForm = document.getElementById('login_form');
    if (show) {
        loginForm.style.display = 'block';
    } else {
        loginForm.style.display = 'none';
    }
}

function getUuid() {
    var http = new XMLHttpRequest();
    http.open('GET', '/fn/uuid');
    http.send();
    http.onreadystatechange = function () {
        if (http.readyState == 4 && this.status == 200) {
            document.getElementById('uuidField').innerHTML = http.responseText
        }
    }
}
function formatJson() {
    let box = document.getElementById('jsonBox');
    var json = JSON.parse(box.value)
    box.value = JSON.stringify(json, null, 4);
}

function toggleVisible(id) {
    var x = document.getElementById(id);
    if (x.className.indexOf("w3-show") == -1) {
        x.className += " w3-show";
    } else {
        x.className = x.className.replace(" w3-show", "");
    }
}

function login() {
    let user = document.getElementById('username').value;
    let pw = document.getElementById('password').value;

    var object = {};
    object['username'] = user;
    object['password'] = pw;

    fetch('/fn/login', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(object)
    })
        .then(response => response.json())
        .catch(err => console.log('Request failed: ' + err))
        .then(data => {
            showLoginForm(false);
            localStorage.setItem('token', data.token);
            loadNotes();
        });
}

function logout() {
    localStorage.removeItem('token');
    document.getElementById('notes_container').innerHTML = '';
    w3.show('#loginBtn');
    w3.hide('#logoutBtn');
    w3.hide('#notesHeader');
}

function checkToken() {
    let token = localStorage.getItem('token');
    if (token == null) {
        w3.show('#loginBtn');
        w3.hide('#logoutBtn');
        w3.hide('#notesHeader');
        return null;
    } else {
        w3.hide('#loginBtn');
        w3.show('#logoutBtn');
        return token;
    }
}

function loadNotes() {
    let token = checkToken();

    fetch('/fn/notes', {
        method: 'GET',
        headers: {
            'Authorization': 'Bearer ' + token
        }
    })
        .then(response => {
            if (response.status == 401) {
                localStorage.removeItem('token');
                document.getElementById('notes_container').innerHTML = '';
                w3.show('#loginBtn');
                w3.hide('#logoutBtn');
                w3.hide('#notesHeader');
                return '';
            } else {
                w3.show('#notesHeader');
                return response.text();
            }
        })
        .then(data => {
            document.getElementById('notes_container').innerHTML = data;
        });
}

function saveNotes() {
    let noteHtml = document.getElementsByClassName('note');
    var notes = new Array();
    Array.from(noteHtml).forEach(elem => {
        let t = elem.childNodes[1].childNodes[1].innerText;
        let c = elem.childNodes[3].innerText;
        let note = { id: parseInt(elem.dataset.id), creator: elem.dataset.creator, title: t, content: c }
        notes.push(note);
    });

    fetch('/fn/notes', {
        method: 'PUT',
        headers: {
            'Authorization': 'Bearer ' + localStorage.getItem('token'),
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ notes: notes })
    })
        .then(_ => { })
}

function deleteNote(id) {
    fetch('/fn/note/' + id, {
        method: 'DELETE',
        headers: {
            'Authorization': 'Bearer ' + localStorage.getItem('token')
        }
    })
        .then(response => {
            if (response.status == 200) {
                document.getElementById('note-' + id).remove();
            }
        });
}

function createNote() {
    fetch('/fn/note', {
        method: 'POST',
        headers: {
            'Authorization': 'Bearer ' + localStorage.getItem('token')
        }
    })
        .then(response => response.text())
        .then(body => {
            var parser = new DOMParser();
            var doc = parser.parseFromString(body, 'text/html');
            document.getElementById('notes_container').appendChild(doc.querySelector('div'))
        })
        .catch(e => { console.log('error: ' + e) });
}

function saveNote(id) {
    let note_elem = document.getElementById('note-' + id);
    let title = note_elem.childNodes[1].childNodes[1].innerText;
    let content = note_elem.childNodes[3].innerText;
    let note = { id: id, title: title, content: content, creator: note_elem.dataset.creator };
    fetch('/fn/note', {
        method: 'PUT',
        headers: {
            'Authorization': 'Bearer ' + localStorage.getItem('token'),
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(note)
    })
        .then(response => {
            if (response.status == 401) {
                // todo
            }
        })
}