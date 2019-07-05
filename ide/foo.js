
var fooTranscript = null;
var fooEditor = null;
var fooCodeMirror = null;

function fooSetup(transcript, editor) {
    fooTranscript = document.getElementById(transcript);
    fooEditor = document.getElementById(editor);
    fooCodeMirror = CodeMirror.fromTextArea(fooEditor, {
        theme: "idea",
        viewportMargin: 1000,
    });
    document.onkeypress = function(e) {
        if (e.keyCode == 13 && e.ctrlKey) {
            evaluateEditor();
        }
    }
    fooCodeMirror.focus();
}

function scrollDown() {
    var scrollingElement = (document.scrollingElement || document.body);
    scrollingElement.scrollTop = scrollingElement.scrollHeight;
}

function evaluateEditor() {
    var container = document.createElement("div");
    var source = document.createElement("pre");
    var row = document.createElement("div");
    var arrow = document.createElement("span");
    var result = document.createElement("span");

    row.appendChild(arrow);
    row.appendChild(result)
    
    container.appendChild(source);
    container.appendChild(row);
    
    source.className = "cm-s-idea code";
    row.className = "cm-s-idea resultRow";
    arrow.className = "arrow";
    result.className = "result";

    text = fooCodeMirror.getValue().trim();
    fooCodeMirror.setValue("");
    
    source.innerHTML = text;
    result.innerHTML = "(waiting for server...)";
    arrow.innerHTML =  "\u21d2";

    fooTranscript.appendChild(container);

    scrollDown();

    var http = new XMLHttpRequest();
    var params = "source=" + encodeURIComponent(text);
    http.open("POST", "/eval", true);
    http.setRequestHeader('Content-type', 'application/x-www-form-urlencoded');
    http.onreadystatechange = function() {//Call a function when the state changes.
        if(http.readyState == 4 && http.status == 200) {
            result.innerHTML = http.responseText;
        }
        if(http.readyState == 4 && http.status == 500) {
            result.innerHTML = "Failed!";
        }
    }
    http.send(params);
}

