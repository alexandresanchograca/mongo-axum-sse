var eventSource = new EventSource('sse');

eventSource.onmessage = function (event) {
  const userList = JSON.parse(event.data)
  renderTable(userList)
}

function renderTable(userList) {
  const table = document.getElementById('user-list');

  while (table.rows.length > 1) {
    table.deleteRow(1);
  }

  // Render all users from userList
  userList.forEach(user => {
    const row = table.insertRow();
    const emailCell = row.insertCell(0);
    const passwordCell = row.insertCell(1);

    emailCell.textContent = user.email;
    passwordCell.textContent = user.password;
  });
}
