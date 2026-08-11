# The `w3id.org/headwater` namespace

[Q10](../spec/09-decisions.md#q10--naming) fixes `https://w3id.org/headwater/` as the namespace that emitted artifacts carry. The identifier is registered and it resolves. This directory holds the copy of record for what the service serves, and it is not part of the build.

## Why a permanent identifier and not the project domain

A namespace URI inside an emitted artifact is a promise about a name that outlives every release. A domain that the project rents by the year is a weak vehicle for that promise, and the first choice of domain proved the point within a year. [w3id.org](https://w3id.org/) is a permanent identifier service that the W3C Permanent Identifier Community Group runs. It holds the identifier, and it redirects to whatever location the project hosts today. A later change of web address therefore reaches no artifact that any adopter already holds.

The precedent is direct. LinkML publishes its own metamodel at `https://w3id.org/linkml/`, and the [worked example](../evaluations/linkml-worked-example.md) already imports that namespace beside this one.

## The registration

The service took the entry through [perma-id/w3id.org#6533](https://github.com/perma-id/w3id.org/pull/6533), which a maintainer merged on 2026-08-11. The registry now carries a `headwater/` directory that holds the same two files as this one.

A request to `https://w3id.org/headwater/` answers 302 and sends the client to `https://headwater.tools/ns/`. A request that asks for `text/turtle` or `application/rdf+xml` goes to a `.ttl` file under the same path, and every other request goes to the documentation. No vocabulary is published yet, so a request for a `.ttl` answers 404 until an engine emits one.

[`.htaccess`](.htaccess) here is the copy of record, and it matches the file that the registry serves byte for byte. A change to one that does not reach the other makes this directory a lie. To change the live rules, raise another pull request against the registry.

## What is still open

The redirects are temporary (302). Change them to permanent (301) once the web address is settled, which lets clients cache the resolution. That change needs a pull request against the registry and a matching edit here.

The web address is an ordinary domain and the identifier is not. Moving the site later means one edit to the registry entry, and it reaches no artifact that any adopter already holds. That property is the whole reason for the indirection.
