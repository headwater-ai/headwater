# The `w3id.org/headwater` namespace

[Q10](../spec/09-open-questions.md#q10--naming) fixes `https://w3id.org/headwater/` as the namespace that emitted artifacts carry. This directory holds the payload for the registration, and it is not part of the build.

## Why a permanent identifier and not the project domain

A namespace URI inside an emitted artifact is a promise about a name that outlives every release. A domain that the project rents by the year is a weak vehicle for that promise, and the first choice of domain proved the point within a year. [w3id.org](https://w3id.org/) is a permanent identifier service that the W3C Permanent Identifier Community Group runs. It holds the identifier, and it redirects to whatever location the project hosts today. A later change of web address therefore reaches no artifact that any adopter already holds.

The precedent is direct. LinkML publishes its own metamodel at `https://w3id.org/linkml/`, and the [worked example](../evaluations/linkml-worked-example.md) already imports that namespace beside this one.

## How to register it

The service takes registrations as pull requests against [perma-id/w3id.org](https://github.com/perma-id/w3id.org). The fork and the branch already exist, and the branch holds a `headwater/` directory with the two files that the service wants.

- Fork: [jameswbaxter/w3id.org](https://github.com/jameswbaxter/w3id.org), branch `headwater`
- Open the pull request: [compare view](https://github.com/perma-id/w3id.org/compare/master...jameswbaxter:w3id.org:headwater)

A maintainer reviews the request and merges it. The maintainers refuse identifiers that are too generic or that could cause confusion, and `headwater` is specific enough to pass that test. The copy of [`.htaccess`](.htaccess) here is the same file that the branch carries. Change both together, or the record here stops matching what the service serves.

## What has to exist first

The redirect target is `https://headwater.tools/ns/`. Serve something at that path before you open the pull request, because a registration that resolves to nothing gives the reviewer no way to check the request.

The rules send a request for `text/turtle` or `application/rdf+xml` to a `.ttl` file, and every other request to the documentation. The redirects are temporary (302) until the site settles. Change them to permanent (301) after that, which lets clients cache the resolution.
