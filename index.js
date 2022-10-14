const params = new URLSearchParams(window.location.search);
const current_target = params.get("target") || "x86_64-unknown-linux-gnu";

document.title = "Rustup packages availability on " + current_target;

async function processTemplate(target) {
  const source = await fetch("template.html").then((response) =>
    response.text()
  );
  const packages = await fetch("packages.json").then((response) =>
    response.json()
  );
  const additional = await fetch("additional.json").then((response) =>
    response.json()
  );

  const title = [""];
  const oneDayOffset = 1 * 24 * 60 * 60 * 1000;
  for (i of Array(7).keys()) {
    var d = new Date(new Date(additional.datetime) - oneDayOffset * i);
    const date_str = d.toISOString().slice(0, 10);
    title.push(date_str);
  }

  var packages_availability = [];
  for (var pkg of packages) {
    const pkg_data = await fetch(target + "/" + pkg + ".json").then(
      (response) => response.json().catch((_) => {})
    );
    if (pkg_data === undefined || pkg_data === null) {
      continue;
    }
    var availability_list = [];
    for (var t of title) {
      if (!!t) {
        availability_list.push(pkg_data[t]);
      }
    }
    packages_availability.push({
      package_name: pkg,
      availability_list: availability_list,
      last_available: pkg_data.last_available,
    });
  }

  var context = {
    current_target: target,
    title: title,
    packages_availability: packages_availability,
    additional: additional,
  };
  // var source = document.getElementById("entry-template").innerHTML;
  var template = Handlebars.compile(source);
  var html = template(context);
  return html;
}

processTemplate(current_target).then((html) => {
  const body = document.querySelector("body");
  body.innerHTML = html;
});
