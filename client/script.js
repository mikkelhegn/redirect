var redirects;

function init() {
	get_data();
	add_modal_events();
}

function get_data() {
	fetch("/api")
		.then(res => res.json())
		.then(data => {
			redirects = data;
		})
		.then(() => {
			build_table(redirects);
		});
}


function build_table(items) {
	$("#dataTable > tbody").empty();
	// Sort the list before adding to the table
	items.forEach((item) => {
		$("#dataTable > tbody:first").append(`
			<tr id="${item.name}">  
				<td>
					${item.name}
				</td>
				<td>
					${item.url}
				</td>
				<td>
					<a href="${location.origin + '?' + item.short_url}" class="link-light">${location.origin + '?' + item.short_url}
				</td> 
				<td>
					<a href="#" id="${item.short_url}_edit" class="btn btn-success edit-btn" data-key="${item.short_url}">
						Edit
					</a>
					<a href="#" id="${item.short_url}_delete" class="btn btn-danger delete-btn" data-key="${item.short_url}">
						Delete
					</a>
								<a href="#" id="${item.short_url}_qr" class="btn btn-secondary" data-key="${location.origin + '?' + item.short_url}">
						QR
					</a>
				</td>
			</tr>
		`);

		$(`#${item.short_url}_delete`).click(function() {
			var key = $(this).data("key");
			fetch(`/api?${key}`, {
				method: 'DELETE',
			})
				.then(() => {
					get_data();
				})
		});

		$(`#${item.short_url}_edit`).click(function() {
			var key = $(this).data("key");
			fetch(`/api?${key}`)
				.then((response) => response.json())
				.then((item) => {
					$("#editName").val(item.name);
					$("#editDestination").val(item.url);
					$("#editShortUrl").append(`
						<a href="${location.origin + '?' + item.short_url}" data="${item.short_url}" class="link-dark"> 
						${location.origin + '?' + item.short_url}
						</a > `);
					$("#editModal").modal("show");
				});
		});

		$(`#${item.short_url}_qr`).click(function() {
			var key = $(this).data("key");
			const encoded_url = encodeURIComponent(key);
			fetch(`/qr?${encoded_url}`)
				.then((res) => res.text())
				.then((text) => {
					$("#qrModalBody").empty();
					$("#qrModalBody").append(text);
					$("#qrModal").modal("show");
				});
		});
	});
}

function save_item() {
	var data = {
		name: $("#editName").val(),
		url: $("#editDestination").val(),
		short_url: $("#editShortUrl a").attr("data")
	}
	fetch('/api', {
		method: 'POST',
		body: JSON.stringify(data)
	})
		.then(res => {
			if (res.status != 201) {
				throw new Error("Failed to submit data");
			} else {
				get_data();
				$("#editModal").modal("hide");
			}
		})
}

function add_modal_events() {
	$('#editModal')[0].addEventListener('hidden.bs.modal', function() {
		if ($('#editShortUrl a').length > 0) {
			$('#editShortUrl a')[0].remove();
		}
		$('#editForm')[0].reset();
	})
}

